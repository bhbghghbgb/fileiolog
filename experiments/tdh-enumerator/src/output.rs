use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::sync::mpsc;
use std::thread;

pub use crate::types::WriteCommand;
use crate::types::{EventObservation, EventTypeInfo};

/// Channel buffer size for write commands
const WRITE_CHANNEL_BUFFER: usize = 4096;

/// Pre-allocated capacity for observations per event type
#[allow(dead_code)]
const OBSERVATIONS_PER_TYPE: usize = 4096;

/// Handles writing event data to disk on a dedicated thread.
///
/// Architecture:
/// - Receives WriteCommand messages via a channel
/// - Writes one file per event type for interruptibility (partial results survive crashes)
/// - On Shutdown, writes a final combined summary file
pub struct DiskWriter {
    tx: mpsc::SyncSender<WriteCommand>,
    join_handle: Option<thread::JoinHandle<()>>,
}

impl DiskWriter {
    pub fn new(output_prefix: &str) -> Self {
        let (tx, rx) = mpsc::sync_channel(WRITE_CHANNEL_BUFFER);
        let prefix = output_prefix.to_string();

        let join_handle = thread::Builder::new()
            .name("disk-writer".into())
            .spawn(move || {
                Self::writer_thread(&prefix, rx);
            })
            .expect("Failed to spawn disk writer thread");

        Self {
            tx,
            join_handle: Some(join_handle),
        }
    }

    /// Send a command to the writer thread (non-blocking)
    pub fn send(&self, cmd: WriteCommand) {
        // Ignore send errors (channel closed = writer shutting down)
        let _ = self.tx.try_send(cmd);
    }

    /// Shutdown the writer thread and write final summary
    pub fn shutdown(mut self) {
        let _ = self.tx.send(WriteCommand::Shutdown);
        if let Some(handle) = self.join_handle.take() {
            let _ = handle.join();
        }
    }

    fn writer_thread(output_prefix: &str, rx: mpsc::Receiver<WriteCommand>) {
        // Per-type file writers, opened lazily
        let mut type_writers: HashMap<String, BufWriter<File>> = HashMap::new();
        // Accumulate type info for the final summary
        let mut type_infos: Vec<EventTypeInfo> = Vec::with_capacity(256);
        // Count observations per type
        let mut observation_counts: HashMap<String, u64> = HashMap::new();

        // Ensure output directory exists
        let _ = fs::create_dir_all(output_prefix);

        loop {
            match rx.recv() {
                Ok(WriteCommand::NewType(type_info)) => {
                    let key = Self::type_file_key(&type_info);

                    // Write schema file immediately (small, important)
                    if let Err(e) = Self::write_type_schema_file(output_prefix, &key, &type_info) {
                        log::error!("Failed to write type schema file {}: {}", key, e);
                    }

                    type_infos.push(type_info);
                    observation_counts.entry(key).or_insert(0);
                }
                Ok(WriteCommand::Observation(obs)) => {
                    *observation_counts.entry(obs.type_key.clone()).or_insert(0) += 1;

                    // Append to per-type file
                    match type_writers.entry(obs.type_key.clone()) {
                        std::collections::hash_map::Entry::Occupied(mut e) => {
                            Self::append_observation(e.get_mut(), &obs);
                        }
                        std::collections::hash_map::Entry::Vacant(e) => {
                            let path = format!(
                                "{}/{}_observations.jsonl",
                                output_prefix,
                                Self::sanitize_key(&obs.type_key)
                            );
                            match File::create(&path) {
                                Ok(f) => {
                                    let mut w = BufWriter::with_capacity(64 * 1024, f);
                                    Self::append_observation(&mut w, &obs);
                                    e.insert(w);
                                }
                                Err(e) => {
                                    log::error!("Failed to create {}: {}", path, e);
                                }
                            }
                        }
                    }
                }
                Ok(WriteCommand::Shutdown) => {
                    break;
                }
                Err(_) => {
                    // Channel closed, treat as shutdown
                    break;
                }
            }
        }

        // Flush and close all per-type writers
        for (key, mut writer) in type_writers {
            if let Err(e) = writer.flush() {
                log::error!("Failed to flush writer for {}: {}", key, e);
            }
        }

        // Write final summary file
        if let Err(e) = Self::write_summary_file(output_prefix, &type_infos, &observation_counts) {
            log::error!("Failed to write summary file: {}", e);
        }

        log::info!(
            "Disk writer shutting down. {} event types, total observations written.",
            type_infos.len()
        );
    }

    fn type_file_key(info: &EventTypeInfo) -> String {
        format!(
            "{}_{}_v{}_op{}",
            Self::sanitize_key(&info.provider_name),
            info.event_id,
            info.version,
            info.opcode,
        )
    }

    fn sanitize_key(s: &str) -> String {
        s.chars()
            .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
            .collect()
    }

    fn write_type_schema_file(
        output_prefix: &str,
        key: &str,
        type_info: &EventTypeInfo,
    ) -> Result<(), String> {
        let path = format!("{}/{}_schema.json", output_prefix, Self::sanitize_key(key));
        let file = File::create(&path).map_err(|e| format!("create: {}", e))?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, type_info)
            .map_err(|e| format!("json: {}", e))?;
        writer.flush().map_err(|e| format!("flush: {}", e))?;
        Ok(())
    }

    fn append_observation(writer: &mut BufWriter<File>, obs: &EventObservation) {
        // Write as JSONL (one JSON object per line) for efficient appending
        if let Ok(mut line) = serde_json::to_vec(obs) {
            line.push(b'\n');
            let _ = writer.write_all(&line);
        }
    }

    fn write_summary_file(
        output_prefix: &str,
        type_infos: &[EventTypeInfo],
        observation_counts: &HashMap<String, u64>,
    ) -> Result<(), String> {
        let path = format!("{}/_summary.json", output_prefix);
        let file = File::create(&path).map_err(|e| format!("create: {}", e))?;
        let mut writer = BufWriter::new(file);

        #[derive(serde::Serialize)]
        struct TypeSummary<'a> {
            #[serde(flatten)]
            type_info: &'a EventTypeInfo,
            observation_count: u64,
        }

        let summaries: Vec<TypeSummary> = type_infos
            .iter()
            .map(|ti| {
                let key = Self::type_file_key(ti);
                let count = observation_counts.get(&key).copied().unwrap_or(0);
                TypeSummary {
                    type_info: ti,
                    observation_count: count,
                }
            })
            .collect();

        serde_json::to_writer_pretty(&mut writer, &summaries)
            .map_err(|e| format!("json: {}", e))?;
        writer.flush().map_err(|e| format!("flush: {}", e))?;

        log::info!("Wrote summary with {} event types to {}", summaries.len(), path);
        Ok(())
    }
}
