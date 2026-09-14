//! Small wrapper around Google Cloud Text-to-Speech with retry-on-defect.
//!
//! Google's TTS occasionally returns silent or truncated audio for short
//! utterances. This crate wraps the synthesize call with a retry loop and
//! reports whether the final audio passed defect checks or whether all
//! attempts were defective and we returned the last one anyway.
//!
//! The crate is application-agnostic — callers map their own language enums
//! into the `language_code` + `voice_name` strings the Google API takes.

use anyhow::{Context, Result};
use base64::Engine;
use serde::{Deserialize, Serialize};

pub mod gemini;
mod telemetry;

pub use telemetry::{RequestCounts, request_counts};

#[derive(Debug, Clone)]
pub struct GoogleTtsRequest {
    pub text: String,
    /// BCP-47 language tag, e.g. `"fr-FR"`.
    pub language_code: String,
    /// Google TTS voice name, e.g. `"fr-FR-Chirp3-HD-Achernar"`.
    pub voice_name: String,
    /// Playback speed multiplier (1.0 = normal).
    pub speed: f64,
    /// Treat `text` as SSML rather than plain text.
    pub is_ssml: bool,
}

/// What we got back from the API after the retry loop, plus a status
/// indicating whether defect checks passed. We *always* return audio bytes if
/// the API call succeeded — `status` lets the caller decide what to do when
/// every attempt was flagged as defective.
#[derive(Debug, Clone)]
pub struct GoogleTtsOutcome {
    /// OGG/Opus-encoded audio bytes (the encoding we always request).
    pub audio_bytes: Vec<u8>,
    /// Total attempts made, including the one whose audio we returned.
    pub attempts: usize,
    /// `Passed` if the returned audio passed defect checks. `HitLimit` if
    /// every attempt was flagged and we returned the last one regardless.
    pub status: TtsStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TtsStatus {
    /// Returned audio passed `audio_defect` after `attempts` tries.
    Passed,
    /// Hit the retry limit; the returned audio is whatever the last attempt
    /// produced, and `last_defect` is what flagged it.
    HitLimit { last_defect: &'static str },
}

impl TtsStatus {
    pub fn passed(&self) -> bool {
        matches!(self, TtsStatus::Passed)
    }
}

#[derive(Debug, Clone)]
pub struct GoogleTtsClient {
    api_key: String,
    http: reqwest::Client,
    max_attempts: usize,
}

impl GoogleTtsClient {
    pub fn new(api_key: String) -> Self {
        Self::with_http(api_key, reqwest::Client::new())
    }

    pub fn with_http(api_key: String, http: reqwest::Client) -> Self {
        Self {
            api_key,
            http,
            max_attempts: 5,
        }
    }

    /// Override the retry budget. Defaults to 5.
    pub fn with_max_attempts(mut self, n: usize) -> Self {
        self.max_attempts = n.max(1);
        self
    }

    /// Call Google TTS with the retry-on-defect loop. Returns once we either
    /// get audio that passes [`audio_defect`] or exhaust `max_attempts`.
    pub async fn synthesize(&self, request: &GoogleTtsRequest) -> Result<GoogleTtsOutcome> {
        let url = "https://texttospeech.googleapis.com/v1beta1/text:synthesize";

        let input = if request.is_ssml {
            GoogleTtsInput {
                text: None,
                ssml: Some(request.text.clone()),
            }
        } else {
            GoogleTtsInput {
                text: Some(request.text.clone()),
                ssml: None,
            }
        };
        let payload = GoogleTtsRequestBody {
            input,
            voice: GoogleTtsVoice {
                language_code: request.language_code.clone(),
                name: request.voice_name.clone(),
            },
            audio_config: GoogleTtsAudioConfig {
                audio_encoding: "OGG_OPUS".to_string(),
                speaking_rate: request.speed,
            },
        };

        let mut last_bytes: Vec<u8> = Vec::new();
        let mut last_defect: Option<&'static str> = None;
        let mut attempts = 0usize;

        while attempts < self.max_attempts {
            attempts += 1;
            let body = self.post(url, &payload).await?;
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(&body.audio_content)
                .context("Google TTS audio_content was not valid base64")?;

            match audio_defect(&bytes) {
                None => {
                    return Ok(GoogleTtsOutcome {
                        audio_bytes: bytes,
                        attempts,
                        status: TtsStatus::Passed,
                    });
                }
                Some(defect) => {
                    log::warn!(
                        "google-tts: defective audio ({defect}) on attempt {attempts}/{}, \
                         retrying",
                        self.max_attempts
                    );
                    last_defect = Some(defect);
                    last_bytes = bytes;
                }
            }
        }

        Ok(GoogleTtsOutcome {
            audio_bytes: last_bytes,
            attempts,
            status: TtsStatus::HitLimit {
                last_defect: last_defect.unwrap_or("unknown"),
            },
        })
    }
}

/// Rate limits and server errors are retried with exponential backoff (about
/// two minutes in total) and don't count against the defect budget: a
/// per-minute quota blip must not kill a run that has already synthesized
/// hundreds of clips.
const TRANSIENT_ATTEMPTS: u32 = 8;

impl GoogleTtsClient {
    async fn post(
        &self,
        url: &str,
        payload: &GoogleTtsRequestBody,
    ) -> Result<GoogleTtsResponseBody> {
        for attempt in 1..=TRANSIENT_ATTEMPTS {
            // The key travels in a header, not the query string, so a
            // transport error's URL never carries it into a log.
            telemetry::record_request(telemetry::Backend::Chirp3);
            let response = self
                .http
                .post(url)
                .header("X-Goog-Api-Key", &self.api_key)
                .header("Content-Type", "application/json")
                .json(payload)
                .send()
                .await
                .context("Google TTS request failed")?;
            let status = response.status();
            if status.is_success() {
                return response
                    .json()
                    .await
                    .context("Failed to parse Google TTS response JSON");
            }
            let body = response.text().await.unwrap_or_default();
            let transient =
                status == reqwest::StatusCode::TOO_MANY_REQUESTS || status.is_server_error();
            if !transient || attempt == TRANSIENT_ATTEMPTS {
                anyhow::bail!("Google TTS error ({status}): {body}");
            }
            let delay = std::time::Duration::from_secs(1 << (attempt - 1));
            log::warn!(
                "google-tts: {status} on attempt {attempt}/{TRANSIENT_ATTEMPTS}, retrying in {}s",
                delay.as_secs()
            );
            tokio::time::sleep(delay).await;
        }
        unreachable!("the last attempt returns or bails")
    }
}

// --- request/response wire types (private — exposed via GoogleTtsRequest) ---

#[derive(Serialize)]
struct GoogleTtsRequestBody {
    input: GoogleTtsInput,
    voice: GoogleTtsVoice,
    #[serde(rename = "audioConfig")]
    audio_config: GoogleTtsAudioConfig,
}

#[derive(Serialize)]
struct GoogleTtsInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ssml: Option<String>,
}

#[derive(Serialize)]
struct GoogleTtsVoice {
    #[serde(rename = "languageCode")]
    language_code: String,
    name: String,
}

#[derive(Serialize)]
struct GoogleTtsAudioConfig {
    #[serde(rename = "audioEncoding")]
    audio_encoding: String,
    #[serde(rename = "speakingRate")]
    speaking_rate: f64,
}

#[derive(Deserialize)]
struct GoogleTtsResponseBody {
    #[serde(rename = "audioContent")]
    audio_content: String,
}

// --- audio defect detection -------------------------------------------------

/// Returns `Some(reason)` if the audio looks defective enough to warrant a
/// retry, or `None` if it's acceptable. Only short clips (<1s) are inspected
/// — longer outputs are assumed fine, since the failure modes we care about
/// (silence, truncation) show up in short utterances.
pub fn audio_defect(audio_bytes: &[u8]) -> Option<&'static str> {
    let (samples, sample_rate) = match decode_audio_to_f32(audio_bytes) {
        Ok((s, sr)) if !s.is_empty() && sr > 0 => (s, sr),
        _ => return Some("failed to decode"),
    };
    samples_defect(&samples, sample_rate)
}

/// Same checks as [`audio_defect`] but on pre-decoded mono f32 samples.
/// Use this when the caller already has the audio in PCM form (e.g.
/// decoded via ffmpeg for WAV input).
pub fn samples_defect(samples: &[f32], sample_rate: u32) -> Option<&'static str> {
    if samples.is_empty() || sample_rate == 0 {
        return Some("failed to decode");
    }

    let duration_s = samples.len() as f64 / sample_rate as f64;
    if duration_s >= 1.0 {
        return None;
    }

    // Silence check: real speech runs -20 to -10 dB RMS; broken Google TTS
    // sits near -50 dB. 0.01 (-40 dB) sits cleanly between the two, and
    // using RMS (not peak) keeps us robust to isolated clicks in
    // otherwise-silent output.
    let sum_sq: f64 = samples.iter().map(|&s| (s as f64) * (s as f64)).sum();
    let rms = (sum_sq / samples.len() as f64).sqrt() as f32;
    if rms < 0.01 {
        return Some("silent");
    }

    // Tail-decay check for very short clips: a full utterance fades 50-70 dB
    // from its peak to the end of the clip; a truncated one barely drops.
    if duration_s < 0.5 && tail_decay_db(samples, sample_rate).is_some_and(|d| d < 15.0) {
        return Some("cut off");
    }

    None
}

/// Estimates how many dB the signal drops between its loudest frame and the
/// end of the clip, using the slope of the last 100 ms of the RMS envelope.
/// Returns `None` if the clip is too short to build a meaningful envelope.
fn tail_decay_db(samples: &[f32], sample_rate: u32) -> Option<f32> {
    const HOP_S: f32 = 0.010;
    const WIN_S: f32 = 0.020;
    const TAIL_FRAMES: usize = 10; // 100 ms at 10 ms hop

    let win = (sample_rate as f32 * WIN_S) as usize;
    let hop = (sample_rate as f32 * HOP_S) as usize;
    if win == 0 || hop == 0 || samples.len() < win {
        return None;
    }

    let mut envelope = Vec::new();
    let mut start = 0;
    while start + win <= samples.len() {
        let slice = &samples[start..start + win];
        let sum_sq: f64 = slice.iter().map(|&s| (s as f64) * (s as f64)).sum();
        envelope.push((sum_sq / slice.len() as f64).sqrt() as f32);
        start += hop;
    }
    if envelope.len() < TAIL_FRAMES {
        return None;
    }

    let tail_db: Vec<f32> = envelope[envelope.len() - TAIL_FRAMES..]
        .iter()
        .map(|&r| 20.0 * r.max(1e-6).log10())
        .collect();
    let n = tail_db.len() as f32;
    let mean_x = (n - 1.0) / 2.0 * HOP_S;
    let mean_y = tail_db.iter().sum::<f32>() / n;
    let mut num = 0.0f32;
    let mut den = 0.0f32;
    for (i, &y) in tail_db.iter().enumerate() {
        let x = i as f32 * HOP_S;
        num += (x - mean_x) * (y - mean_y);
        den += (x - mean_x).powi(2);
    }
    if den == 0.0 {
        return None;
    }
    let slope_db_per_s = num / den;

    let peak_idx = envelope
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))?
        .0;
    let peak_to_end_s = (envelope.len() - 1 - peak_idx) as f32 * HOP_S;
    Some(peak_to_end_s * slope_db_per_s.abs())
}

// --- audio encoders ---------------------------------------------------------

/// 16-bit mono PCM wrapped in a RIFF/WAVE header.
pub fn pcm_to_wav(samples: &[i16], sample_rate: u32) -> Vec<u8> {
    let data_size = (samples.len() * 2) as u32;
    let mut wav = Vec::with_capacity(44 + samples.len() * 2);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_size).to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
    wav.extend_from_slice(&1u16.to_le_bytes()); // mono
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());
    for sample in samples {
        wav.extend_from_slice(&sample.to_le_bytes());
    }
    wav
}

/// 16-bit mono PCM as an Ogg Opus stream (RFC 7845), the container Cloud TTS
/// hands back, so clips from every provider share one format in the packs
/// and the caches. `sample_rate` must be one Opus accepts natively: 8, 12,
/// 16, 24 or 48 kHz.
pub fn encode_ogg_opus(samples: &[i16], sample_rate: u32) -> Result<Vec<u8>> {
    use ogg::writing::{PacketWriteEndInfo, PacketWriter};

    let mut encoder =
        opus::Encoder::new(sample_rate, opus::Channels::Mono, opus::Application::Voip)
            .map_err(|e| anyhow::anyhow!("opus encoder init: {e:?}"))?;
    // The encoder delays its output by its lookahead. Decoders drop that
    // many samples from the front (OpusHead's pre-skip, in 48 kHz units) and
    // trim the end to the final granule position, so the input has to be
    // followed by at least a lookahead of silence for its tail to come out.
    let lookahead = encoder
        .get_lookahead()
        .map_err(|e| anyhow::anyhow!("opus lookahead: {e:?}"))? as usize;
    let to_48k = |n: usize| (n as u64) * 48_000 / sample_rate as u64;
    let pre_skip = to_48k(lookahead);
    let frame = (sample_rate / 50) as usize; // 20 ms
    let frame_48k = to_48k(frame);

    let mut head = Vec::with_capacity(19);
    head.extend_from_slice(b"OpusHead");
    head.push(1); // version
    head.push(1); // channels
    head.extend_from_slice(&(pre_skip as u16).to_le_bytes());
    head.extend_from_slice(&sample_rate.to_le_bytes());
    head.extend_from_slice(&0i16.to_le_bytes()); // output gain
    head.push(0); // channel mapping family

    let vendor = b"yap google-tts";
    let mut tags = Vec::new();
    tags.extend_from_slice(b"OpusTags");
    tags.extend_from_slice(&(vendor.len() as u32).to_le_bytes());
    tags.extend_from_slice(vendor);
    tags.extend_from_slice(&0u32.to_le_bytes());

    let serial = 0x7961_7000; // "yap"
    let mut writer = PacketWriter::new(Vec::new());
    writer
        .write_packet(head, serial, PacketWriteEndInfo::EndPage, 0)
        .context("ogg: writing OpusHead")?;
    writer
        .write_packet(tags, serial, PacketWriteEndInfo::EndPage, 0)
        .context("ogg: writing OpusTags")?;

    let mut padded: Vec<i16> = Vec::with_capacity(samples.len() + lookahead + frame);
    padded.extend_from_slice(samples);
    padded.resize(
        (samples.len() + lookahead).div_ceil(frame).max(1) * frame,
        0,
    );
    let frames = padded.len() / frame;
    let mut packet = vec![0u8; 4000];
    for (i, chunk) in padded.chunks_exact(frame).enumerate() {
        let len = encoder
            .encode(chunk, &mut packet)
            .map_err(|e| anyhow::anyhow!("opus encode: {e:?}"))?;
        let last = i + 1 == frames;
        // Granule positions count every decoded sample, pre-skip included;
        // the final one trims the padding so exactly the input plays.
        let (granule, end) = if last {
            (
                pre_skip + to_48k(samples.len()),
                PacketWriteEndInfo::EndStream,
            )
        } else {
            ((i as u64 + 1) * frame_48k, PacketWriteEndInfo::NormalPacket)
        };
        writer
            .write_packet(packet[..len].to_vec(), serial, end, granule)
            .context("ogg: writing audio packet")?;
    }
    Ok(writer.into_inner())
}

// --- audio decoders ---------------------------------------------------------

/// Dispatches to the right decoder based on magic bytes. Handles the formats
/// we actually see from our TTS providers and pipelines: OGG Opus (Google),
/// WAV (Gemini), and MP3 (OpenAI and others).
pub fn decode_audio_to_f32(bytes: &[u8]) -> Result<(Vec<f32>, u32), String> {
    if bytes.starts_with(b"OggS") {
        decode_ogg_opus_to_f32(bytes)
    } else if bytes.starts_with(b"RIFF") {
        decode_wav_to_f32(bytes)
    } else {
        decode_mp3_to_f32(bytes)
    }
}

/// Decodes WAV (any bit depth hound supports) to mono f32 samples.
pub fn decode_wav_to_f32(bytes: &[u8]) -> Result<(Vec<f32>, u32), String> {
    use std::io::Cursor;

    let mut reader =
        hound::WavReader::new(Cursor::new(bytes)).map_err(|e| format!("wav read error: {e}"))?;
    let spec = reader.spec();
    let channels = spec.channels as usize;

    let interleaved: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .collect::<Result<_, _>>()
            .map_err(|e| format!("wav decode error: {e}"))?,
        hound::SampleFormat::Int => {
            let max = (1i64 << (spec.bits_per_sample - 1)) as f32;
            reader
                .samples::<i32>()
                .map(|s| s.map(|s| s as f32 / max))
                .collect::<Result<_, _>>()
                .map_err(|e| format!("wav decode error: {e}"))?
        }
    };

    let samples: Vec<f32> = interleaved
        .chunks(channels)
        .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
        .collect();

    if samples.is_empty() {
        return Err("No audio data decoded".to_string());
    }

    Ok((samples, spec.sample_rate))
}

pub fn decode_mp3_to_f32(mp3_bytes: &[u8]) -> Result<(Vec<f32>, u32), String> {
    use std::io::Cursor;

    let cursor = Cursor::new(mp3_bytes);
    let mut decoder = minimp3::Decoder::new(cursor);

    let mut samples: Vec<f32> = Vec::new();
    let mut sample_rate = 0u32;

    loop {
        match decoder.next_frame() {
            Ok(frame) => {
                sample_rate = frame.sample_rate as u32;
                let channels = frame.channels;
                for chunk in frame.data.chunks(channels) {
                    let mono = chunk.iter().map(|&s| s as f32).sum::<f32>() / channels as f32;
                    samples.push(mono / 32768.0);
                }
            }
            Err(minimp3::Error::Eof) => break,
            Err(e) => return Err(format!("mp3 decode error: {e:?}")),
        }
    }

    if samples.is_empty() {
        return Err("No audio data decoded".to_string());
    }

    Ok((samples, sample_rate))
}

pub fn decode_ogg_opus_to_f32(bytes: &[u8]) -> Result<(Vec<f32>, u32), String> {
    use std::io::Cursor;

    let mut reader = ogg::PacketReader::new(Cursor::new(bytes));

    // First packet: OpusHead identification header (RFC 7845 §5.1).
    let header = reader
        .read_packet_expected()
        .map_err(|e| format!("ogg read error: {e:?}"))?;
    if !header.data.starts_with(b"OpusHead") || header.data.len() < 19 {
        return Err("not an OpusHead packet".to_string());
    }
    let channels = header.data[9];
    let opus_channels = match channels {
        1 => opus::Channels::Mono,
        2 => opus::Channels::Stereo,
        n => return Err(format!("unsupported channel count: {n}")),
    };

    // Second packet: OpusTags comment header — discard.
    let _tags = reader
        .read_packet_expected()
        .map_err(|e| format!("ogg tags read error: {e:?}"))?;

    // Opus always decodes at 48 kHz internally; pick that as our output rate.
    const DECODE_RATE: u32 = 48_000;
    let mut decoder = opus::Decoder::new(DECODE_RATE, opus_channels)
        .map_err(|e| format!("opus decoder init: {e:?}"))?;

    // Max Opus frame is 120 ms at 48 kHz = 5760 samples per channel.
    let mut frame_buf = vec![0f32; 5760 * channels as usize];
    let mut samples: Vec<f32> = Vec::new();

    while let Some(packet) = reader
        .read_packet()
        .map_err(|e| format!("ogg packet read: {e:?}"))?
    {
        let n = decoder
            .decode_float(&packet.data, &mut frame_buf, false)
            .map_err(|e| format!("opus decode: {e:?}"))?;
        for chunk in frame_buf[..n * channels as usize].chunks(channels as usize) {
            let mono = chunk.iter().sum::<f32>() / channels as f32;
            samples.push(mono);
        }
    }

    if samples.is_empty() {
        return Err("No audio data decoded".to_string());
    }

    Ok((samples, DECODE_RATE))
}

#[cfg(test)]
mod tests {
    use super::*;

    // A synthesized tone survives the Opus round trip with its length and
    // level intact: what the packs store for Gemini clips decodes the same
    // way as what Cloud TTS returns.
    #[test]
    fn ogg_opus_round_trips_pcm() {
        let rate = 24_000u32;
        let len = rate as usize * 3 / 2;
        let samples: Vec<i16> = (0..len)
            .map(|i| {
                ((i as f32 * 440.0 * std::f32::consts::TAU / rate as f32).sin() * 12_000.0) as i16
            })
            .collect();
        let ogg = encode_ogg_opus(&samples, rate).unwrap();
        assert!(ogg.starts_with(b"OggS"));

        // Our decoder neither drops the pre-skip nor trims to the final
        // granule, so apply both here as a player would (RFC 7845 §4.4).
        let head = ogg.windows(8).position(|w| w == b"OpusHead").unwrap();
        let pre_skip = u16::from_le_bytes([ogg[head + 10], ogg[head + 11]]) as usize;
        let (decoded, decoded_rate) = decode_audio_to_f32(&ogg).unwrap();
        assert_eq!(decoded_rate, 48_000);
        let len_48k = len * 48_000 / rate as usize;
        assert!(
            decoded.len() >= pre_skip + len_48k,
            "encoder never flushed its lookahead"
        );
        let played = &decoded[pre_skip..pre_skip + len_48k];
        let rms = |s: &[f32]| {
            (s.iter().map(|x| (*x as f64).powi(2)).sum::<f64>() / s.len() as f64).sqrt()
        };
        assert!((0.2..0.5).contains(&rms(played)), "rms {}", rms(played));
        // The last 5 ms of the input are still a tone, not the encoder's delay line.
        assert!(
            rms(&played[played.len() - 240..]) > 0.2,
            "tail was lost to lookahead"
        );
        assert!(audio_defect(&ogg).is_none());
    }
}
