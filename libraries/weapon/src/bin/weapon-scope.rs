#[cfg(not(feature = "opfs"))]
fn main() {
    eprintln!("weapon-scope requires the 'opfs' feature to be enabled");
    eprintln!("Please run with: cargo run --bin weapon-scope --features opfs");
    std::process::exit(1);
}

#[cfg(feature = "opfs")]
fn main() {
    use std::collections::BTreeMap;
    use std::fs::File;
    use std::io::Read;
    use std::path::PathBuf;
    use weapon::opfs::{EventLogRecord, parse_device_counts, parse_event_log_records};
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <path-to-event-log-file>", args[0]);
        eprintln!("\nExample: {} ./events.blob", args[0]);
        std::process::exit(1);
    }

    let file_path = PathBuf::from(&args[1]);

    if !file_path.exists() {
        eprintln!("Error: File '{}' does not exist", file_path.display());
        std::process::exit(1);
    }

    let mut file = match File::open(&file_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error opening file '{}': {}", file_path.display(), e);
            std::process::exit(1);
        }
    };

    let mut bytes = Vec::new();
    if let Err(e) = file.read_to_end(&mut bytes) {
        eprintln!("Error reading file '{}': {}", file_path.display(), e);
        std::process::exit(1);
    }

    println!("WeaponScope - OPFS Event Log Analyzer");
    println!("=====================================");
    println!("File: {}", file_path.display());
    println!(
        "Size: {} bytes ({:.2} KB)",
        bytes.len(),
        bytes.len() as f64 / 1024.0
    );
    println!();

    // Parse device counts
    println!("Device Counts:");
    println!("--------------");
    let device_counts = parse_device_counts(&bytes);

    if device_counts.is_empty() {
        println!("  No devices found or invalid file format");
    } else {
        let total_events: usize = device_counts.values().sum();
        println!("  Total devices: {}", device_counts.len());
        println!("  Total events: {total_events}");
        println!();

        for (device_id, count) in &device_counts {
            println!("  Device: {device_id}");
            println!("    Events: {count}");
        }
    }

    println!();
    println!("Event Details:");
    println!("--------------");

    // Parse full event records
    let records = parse_event_log_records(&bytes);

    if records.is_empty() {
        println!("  No events found or invalid file format");
    } else {
        // Group records by device
        let mut events_by_device: BTreeMap<String, Vec<&EventLogRecord>> = BTreeMap::new();
        for record in &records {
            events_by_device
                .entry(record.device_id.clone())
                .or_default()
                .push(record);
        }

        println!("  Total parsed events: {}", records.len());
        println!();

        for (device_id, device_records) in events_by_device {
            println!("  Device: {device_id}");
            println!("  -------");

            // Check for index continuity
            let mut expected_index = 0;
            let mut has_gaps = false;
            let mut has_backtracking = false;

            for (i, record) in device_records.iter().enumerate() {
                let index = record.within_device_events_index;

                if index < expected_index {
                    has_backtracking = true;
                    println!(
                        "    ⚠️  Event {i}: Index {index} (BACKTRACKING - expected >= {expected_index})"
                    );
                } else if index > expected_index {
                    has_gaps = true;
                    println!("    ⚠️  Event {i}: Index {index} (GAP - expected {expected_index})");
                } else {
                    println!("    Event {i}: Index {index}");
                }

                // Show timestamp
                println!("      Timestamp: {}", record.event.timestamp);

                // Show a preview of the event data (first 100 chars)
                let event_str = serde_json::to_string(&record.event.event)
                    .unwrap_or_else(|_| "Invalid JSON".to_string());
                let preview = if event_str.len() > 100 {
                    // Safely truncate at a character boundary
                    let mut end = 100;
                    while !event_str.is_char_boundary(end) && end > 0 {
                        end -= 1;
                    }
                    format!("{}...", &event_str[..end])
                } else {
                    event_str
                };
                println!("      Data: {preview}");

                expected_index = index + 1;
            }

            if has_gaps {
                println!("    ⚠️  WARNING: This device has gaps in event indices");
            }
            if has_backtracking {
                println!("    ❌ ERROR: This device has backtracking (indices going backwards)");
            }
            if !has_gaps && !has_backtracking {
                println!("    ✅ All indices are sequential");
            }

            println!();
        }
    }

    println!();
    println!("Summary:");
    println!("--------");

    // Check if parsed counts match
    let mut parsed_device_counts: BTreeMap<String, usize> = BTreeMap::new();
    for record in &records {
        *parsed_device_counts
            .entry(record.device_id.clone())
            .or_default() += 1;
    }

    let mut has_mismatch = false;
    for (device_id, expected_count) in &device_counts {
        let actual_count = parsed_device_counts.get(device_id).copied().unwrap_or(0);
        if actual_count != *expected_count {
            println!(
                "  ❌ Count mismatch for device {device_id}: header says {expected_count} but found {actual_count} events"
            );
            has_mismatch = true;
        }
    }

    if !has_mismatch {
        println!("  ✅ All device counts match between header and parsed events");
    }

    // Check for devices in parsed events but not in counts
    for (device_id, count) in &parsed_device_counts {
        if !device_counts.contains_key(device_id) {
            println!("  ⚠️  Device {device_id} has {count} events but is not in device counts");
        }
    }
}
