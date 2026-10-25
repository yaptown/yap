//! Decode and encode provider containers to f32/i16 PCM, plus signal sanity checks.
//! Provider-agnostic audio helpers.

#[cfg(any(feature = "full", feature = "mp3-encode"))]
use anyhow::Result;

// --- audio defect detection -------------------------------------------------

/// Returns `Some(reason)` if the audio looks defective enough to warrant a
/// retry, or `None` if it's acceptable. Only short clips (<1s) are inspected
/// — longer outputs are assumed fine, since the failure modes we care about
/// (silence, truncation) show up in short utterances.
#[cfg(feature = "full")]
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

/// Mono speech at 48 kbps. Server/offline only: LAME must not ship in clients.
#[cfg(feature = "mp3-encode")]
pub fn encode_mp3(samples: &[i16], sample_rate: u32) -> Result<Vec<u8>> {
    use mp3lame_encoder::{Bitrate, Builder, FlushGap, MonoPcm, Quality};
    let mut encoder = Builder::new()
        .ok_or_else(|| anyhow::anyhow!("LAME init failed"))?
        .with_num_channels(1)?
        .with_sample_rate(sample_rate)?
        .with_brate(Bitrate::Kbps48)?
        .with_quality(Quality::Best)?
        .build()?;
    let mut bytes = Vec::with_capacity(mp3lame_encoder::max_required_buffer_size(samples.len()));
    encoder.encode_to_vec(MonoPcm(samples), &mut bytes)?;
    bytes.reserve(7200);
    encoder.flush_to_vec::<FlushGap>(&mut bytes)?;
    Ok(bytes)
}

// --- audio decoders ---------------------------------------------------------

/// Dispatches to the right decoder based on magic bytes. Handles the formats
/// we actually see from our TTS providers and pipelines: OGG Opus (recordings/old caches),
/// WAV (raw synthesis), and MP3 (published audio).
#[cfg(feature = "full")]
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
#[cfg(feature = "full")]
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

#[cfg(feature = "full")]
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

#[cfg(feature = "full")]
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

#[cfg(all(test, feature = "full", feature = "mp3-encode"))]
mod tests {
    #[test]
    fn mp3_round_trips_speech_rate_pcm() {
        let rate = 24_000;
        let samples: Vec<i16> = (0..rate)
            .map(|i| {
                ((i as f32 * 440.0 * std::f32::consts::TAU / rate as f32).sin() * 12_000.0) as i16
            })
            .collect();
        let bytes = super::encode_mp3(&samples, rate).unwrap();
        let (decoded, decoded_rate) = super::decode_mp3_to_f32(&bytes).unwrap();
        assert_eq!(decoded_rate, rate);
        assert!((24_000..27_000).contains(&decoded.len()));
        assert!(bytes.len() < 8_000);
        assert!(super::audio_defect(&bytes).is_none());
    }
}
