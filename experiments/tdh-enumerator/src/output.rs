use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::{Arc, Mutex};

use crate::types::EventInfo;

/// Accumulates events and writes output files
pub struct OutputWriter {
    events: Arc<Mutex<Vec<EventInfo>>>,
    output_prefix: String,
}

impl OutputWriter {
    pub fn new(output_prefix: &str) -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
            output_prefix: output_prefix.to_string(),
        }
    }

    pub fn event_callback(&self) -> impl Fn(EventInfo) + Clone + Send + Sync + 'static {
        let events = self.events.clone();
        move |event: EventInfo| {
            // Console output (human-readable)
            print_event_summary(&event);

            // Store for file output
            if let Ok(mut evts) = events.lock() {
                evts.push(event);
            }
        }
    }

    pub fn write_files(&self) -> Result<(), String> {
        let events = self.events.lock().map_err(|e| format!("Lock error: {}", e))?;

        // Write JSON file
        let json_path = format!("{}.json", self.output_prefix);
        let json_file = File::create(&json_path)
            .map_err(|e| format!("Failed to create {}: {}", json_path, e))?;
        let mut json_writer = BufWriter::new(json_file);

        serde_json::to_writer_pretty(&mut json_writer, &*events)
            .map_err(|e| format!("JSON write error: {}", e))?;
        json_writer
            .flush()
            .map_err(|e| format!("JSON flush error: {}", e))?;

        log::info!("Wrote {} events to {}", events.len(), json_path);

        // Write human-readable text file
        let txt_path = format!("{}.txt", self.output_prefix);
        let txt_file = File::create(&txt_path)
            .map_err(|e| format!("Failed to create {}: {}", txt_path, e))?;
        let mut txt_writer = BufWriter::new(txt_file);

        for event in events.iter() {
            write_event_text(&mut txt_writer, event)
                .map_err(|e| format!("Text write error: {}", e))?;
        }
        txt_writer
            .flush()
            .map_err(|e| format!("Text flush error: {}", e))?;

        log::info!("Wrote {} events to {}", events.len(), txt_path);

        Ok(())
    }

    pub fn event_count(&self) -> usize {
        self.events.lock().map(|e| e.len()).unwrap_or(0)
    }
}

fn print_event_summary(event: &EventInfo) {
    println!(
        "[{}] {} (opcode={}, v{}, id={}) PID={} TID={} | {} properties",
        event.provider_name,
        event.opcode_name,
        event.opcode,
        event.version,
        event.event_id,
        event.process_id,
        event.thread_id,
        event.properties.len()
    );
}

fn write_event_text(w: &mut impl Write, event: &EventInfo) -> std::io::Result<()> {
    writeln!(w, "════════════════════════════════════════════════════════════════")?;
    writeln!(w, "Provider:         {}", event.provider_name)?;
    writeln!(w, "Provider GUID:    {}", event.provider_guid)?;
    writeln!(w, "Event ID:         {}", event.event_id)?;
    writeln!(w, "Opcode:           {} ({})", event.opcode_name, event.opcode)?;
    writeln!(w, "Version:          {}", event.version)?;
    writeln!(w, "Level:            {}", event.level)?;
    writeln!(w, "Keyword:          0x{:016x}", event.keyword)?;
    writeln!(w, "Process ID:       {}", event.process_id)?;
    writeln!(w, "Thread ID:        {}", event.thread_id)?;
    writeln!(w, "Timestamp:        {}", event.timestamp)?;
    writeln!(w, "Activity ID:      {}", event.activity_id)?;
    writeln!(w, "Task Name:        {}", event.task_name)?;
    writeln!(w, "Decoding Source:  {}", event.decoding_source)?;
    writeln!(w, "User Data Size:   {} bytes", event.user_data_length)?;
    writeln!(w, "")?;
    writeln!(w, "Properties ({}):", event.properties.len())?;

    for (i, prop) in event.properties.iter().enumerate() {
        writeln!(w, "  [{}] {}:", i, prop.name)?;
        writeln!(w, "      InType:     {} ({})", prop.in_type_name, prop.in_type)?;
        writeln!(w, "      OutType:    {} ({})", prop.out_type_name, prop.out_type)?;
        writeln!(w, "      Flags:      {}", prop.flags_hex)?;
        match &prop.length {
            crate::types::PropertyLengthInfo::Fixed(l) => {
                writeln!(w, "      Length:     {} bytes", l)?;
            }
            crate::types::PropertyLengthInfo::Index(idx) => {
                writeln!(w, "      Length:     index {}", idx)?;
            }
        }
        if let Some(count) = &prop.count {
            match count {
                crate::types::PropertyCountInfo::Fixed(c) => {
                    writeln!(w, "      Count:      {}", c)?;
                }
                crate::types::PropertyCountInfo::Index(idx) => {
                    writeln!(w, "      Count:      index {}", idx)?;
                }
            }
        }
    }

    writeln!(w, "")?;
    writeln!(w, "User Data (hex):")?;
    // Print hex dump in rows of 16 bytes
    let hex_str = &event.user_data_hex;
    let bytes: Vec<u8> = (0..hex_str.len())
        .step_by(2)
        .filter_map(|i| {
            hex_str
                .get(i..i + 2)
                .and_then(|h| u8::from_str_radix(h, 16).ok())
        })
        .collect();

    for (offset, chunk) in bytes.chunks(16).enumerate() {
        let hex_part: String = chunk
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(" ");
        let ascii_part: String = chunk
            .iter()
            .map(|&b| if b >= 0x20 && b < 0x7f { b as char } else { '.' })
            .collect();
        writeln!(
            w,
            "  {:04x}: {:<48}  {}",
            offset * 16,
            hex_part,
            ascii_part
        )?;
    }

    writeln!(w, "")?;
    Ok(())
}
