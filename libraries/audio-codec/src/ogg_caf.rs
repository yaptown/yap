//! Lossless Ogg Opus → Core Audio Format container remuxing. This module has
//! no decoder dependency and is available on native and wasm targets alike.
use anyhow::{Context, Result, ensure};
use ogg::PacketReader;
use std::io::Cursor;

/// Preserve Opus packets verbatim in a CAF container, which Apple's native
/// players support (unlike Ogg). Supports the mono/stereo mapping used by TTS.
pub fn ogg_opus_to_caf(ogg: &[u8]) -> Result<Vec<u8>> {
    let mut reader = PacketReader::new(Cursor::new(ogg));
    let head = reader.read_packet_expected().context("reading OpusHead")?;
    ensure!(
        head.data.len() >= 19 && head.data.starts_with(b"OpusHead"),
        "missing OpusHead"
    );
    ensure!(head.data[8] <= 15, "unsupported OpusHead version");
    let channels = head.data[9];
    ensure!(
        (1..=2).contains(&channels) && head.data[18] == 0,
        "unsupported Opus channel mapping"
    );
    let pre_skip = u16::from_le_bytes([head.data[10], head.data[11]]) as u32;
    // The input-rate field is informational. All Opus timing is in 48 kHz units.
    let _input_rate = u32::from_le_bytes(head.data[12..16].try_into()?);
    let tags = reader.read_packet_expected().context("reading OpusTags")?;
    ensure!(tags.data.starts_with(b"OpusTags"), "missing OpusTags");
    let mut packets = Vec::new();
    let mut audio = Vec::new();
    let mut total_frames = 0i64;
    // The final page's granule position counts every decoded sample that a
    // player should keep (pre-skip included); anything the packets carry
    // beyond it is encoder padding (RFC 7845 §4.4).
    let mut final_granule = 0u64;
    while let Some(packet) = reader.read_packet().context("reading Opus packet")? {
        let frames = opus_packet_frames(&packet.data)?;
        packets.push((packet.data.len() as u64, frames));
        total_frames += i64::from(frames);
        final_granule = packet.absgp_page();
        audio.extend_from_slice(&packet.data);
    }
    let packet_count = packets.len() as i64;
    // AVAudioPlayer refuses to start an Opus stream described as having a
    // variable packet duration, so when every packet is the same length (the
    // norm for encoded speech) the description says so and the packet table
    // lists only byte sizes, exactly as Apple's own encoder writes it.
    let frames_per_packet = match packets.first() {
        Some(&(_, frames)) if packets.iter().all(|&(_, f)| f == frames) => frames,
        _ => 0,
    };
    let mut packet_table = Vec::new();
    for &(bytes, frames) in &packets {
        write_varint(&mut packet_table, bytes);
        if frames_per_packet == 0 {
            write_varint(&mut packet_table, u64::from(frames));
        }
    }
    let final_granule = i64::try_from(final_granule).context("granule position")?;
    ensure!(
        packet_count > 0 && final_granule > i64::from(pre_skip) && final_granule <= total_frames,
        "empty or inconsistent Opus audio"
    );
    let valid_frames = final_granule - i64::from(pre_skip);
    let remainder_frames = u32::try_from(total_frames - final_granule).context("padding")?;

    let mut caf = b"caff\x00\x01\x00\x00".to_vec();
    let mut desc = Vec::new();
    desc.extend_from_slice(&48000f64.to_be_bytes());
    desc.extend_from_slice(b"opus");
    for value in [0u32, 0, frames_per_packet, u32::from(channels), 0] {
        desc.extend_from_slice(&value.to_be_bytes());
    }
    write_chunk(&mut caf, b"desc", &desc);
    let mut pakt = Vec::new();
    pakt.extend_from_slice(&packet_count.to_be_bytes());
    pakt.extend_from_slice(&valid_frames.to_be_bytes());
    pakt.extend_from_slice(&pre_skip.to_be_bytes());
    pakt.extend_from_slice(&remainder_frames.to_be_bytes());
    pakt.extend_from_slice(&packet_table);
    write_chunk(&mut caf, b"pakt", &pakt);
    let mut data = 0u32.to_be_bytes().to_vec(); // edit count
    data.extend_from_slice(&audio);
    write_chunk(&mut caf, b"data", &data);
    Ok(caf)
}

fn write_chunk(output: &mut Vec<u8>, name: &[u8; 4], bytes: &[u8]) {
    output.extend_from_slice(name);
    output.extend_from_slice(&(bytes.len() as i64).to_be_bytes());
    output.extend_from_slice(bytes);
}

fn write_varint(output: &mut Vec<u8>, mut value: u64) {
    let mut bytes = [0u8; 10];
    let mut index = bytes.len() - 1;
    bytes[index] = (value & 0x7f) as u8;
    value >>= 7;
    while value > 0 {
        index -= 1;
        bytes[index] = (value & 0x7f) as u8 | 0x80;
        value >>= 7;
    }
    output.extend_from_slice(&bytes[index..]);
}

/// RFC 6716 §3.1: frame duration and frame-count code from the TOC, in samples.
fn opus_packet_frames(packet: &[u8]) -> Result<u32> {
    let toc = *packet.first().context("empty Opus packet")?;
    let config = toc >> 3;
    let samples = if config >= 16 {
        120 << (config & 3)
    } else if config >= 12 {
        480 << (config & 1)
    } else {
        [480, 960, 1920, 2880][usize::from(config & 3)]
    };
    let count = match toc & 3 {
        0 => 1,
        1 | 2 => 2,
        _ => u32::from(packet.get(1).context("missing Opus frame count")? & 0x3f),
    };
    let frames = samples * count;
    ensure!(count > 0 && frames <= 5760, "invalid Opus packet duration");
    Ok(frames)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toc_durations_and_invalid_packets() {
        assert_eq!(opus_packet_frames(&[0]).unwrap(), 480);
        assert_eq!(opus_packet_frames(&[3 << 3]).unwrap(), 2880);
        assert_eq!(opus_packet_frames(&[16 << 3]).unwrap(), 120);
        assert_eq!(opus_packet_frames(&[(19 << 3) | 1]).unwrap(), 1920);
        assert_eq!(opus_packet_frames(&[(16 << 3) | 3, 48]).unwrap(), 5760);
        for invalid in [&[][..], &[3][..], &[3, 0][..], &[3, 63][..]] {
            assert!(opus_packet_frames(invalid).is_err());
        }
        assert!(ogg_opus_to_caf(b"not ogg").is_err());
    }

    #[test]
    fn caf_varints_are_big_endian_seven_bit_groups() {
        let mut bytes = Vec::new();
        for value in [0, 127, 128, 16384] {
            write_varint(&mut bytes, value);
        }
        assert_eq!(bytes, [0, 127, 0x81, 0, 0x81, 0x80, 0]);
    }

    #[cfg(feature = "full")]
    #[test]
    fn real_opus_packets_survive_caf_remux() {
        let pcm: Vec<i16> = (0..4800)
            .map(|i| ((i as f32 * 0.05).sin() * 10000.0) as i16)
            .collect();
        let ogg = crate::encode_ogg_opus(&pcm, 48000).unwrap();
        let caf = ogg_opus_to_caf(&ogg).unwrap();
        assert_eq!(&caf[..8], b"caff\0\x01\0\0");
        let mut chunks = std::collections::BTreeMap::new();
        let mut offset = 8;
        while offset < caf.len() {
            let name = &caf[offset..offset + 4];
            let size =
                i64::from_be_bytes(caf[offset + 4..offset + 12].try_into().unwrap()) as usize;
            chunks.insert(name, &caf[offset + 12..offset + 12 + size]);
            offset += 12 + size;
        }
        let desc = chunks[&b"desc"[..]];
        assert_eq!(desc.len(), 32);
        assert_eq!(&desc[8..12], b"opus");
        let frames_per_packet = u32::from_be_bytes(desc[20..24].try_into().unwrap());
        assert_eq!(frames_per_packet, 960, "libopus emits 20 ms packets");
        let table = chunks[&b"pakt"[..]];
        let packet_count = i64::from_be_bytes(table[..8].try_into().unwrap());
        let mut reader = PacketReader::new(Cursor::new(&ogg));
        let head = reader.read_packet_expected().unwrap();
        let pre_skip = u16::from_le_bytes([head.data[10], head.data[11]]) as i32;
        assert_eq!(
            i32::from_be_bytes(table[16..20].try_into().unwrap()),
            pre_skip
        );
        reader.read_packet_expected().unwrap();
        let mut expected_audio = Vec::new();
        let mut expected_count = 0;
        let mut expected_frames = 0;
        let mut table_offset = 24;
        let read_varint = |offset: &mut usize| {
            let mut value = 0u64;
            loop {
                let byte = table[*offset];
                *offset += 1;
                value = value << 7 | u64::from(byte & 0x7f);
                if byte & 0x80 == 0 {
                    break value;
                }
            }
        };
        while let Some(packet) = reader.read_packet().unwrap() {
            assert_eq!(read_varint(&mut table_offset), packet.data.len() as u64);
            let frames = opus_packet_frames(&packet.data).unwrap();
            // Constant-duration packets carry only their byte size in the table.
            assert_eq!(frames, frames_per_packet);
            expected_frames += i64::from(frames);
            expected_count += 1;
            expected_audio.extend(packet.data);
        }
        assert_eq!(packet_count, expected_count);
        assert_eq!(table_offset, table.len());
        // Exactly the encoded input plays: no lookahead, no trailing padding.
        assert_eq!(i64::from_be_bytes(table[8..16].try_into().unwrap()), 4800);
        assert_eq!(
            u32::from_be_bytes(table[20..24].try_into().unwrap()),
            u32::try_from(expected_frames - i64::from(pre_skip) - 4800).unwrap()
        );
        assert_eq!(&chunks[&b"data"[..]][4..], expected_audio);
    }
}
