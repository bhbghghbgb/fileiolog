mod experiments;
mod fabricate;
mod tdh_helpers;

use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use clap::Parser as ClapParser;

#[derive(Debug, ClapParser)]
#[command(name = "tdh-fabricate")]
#[command(about = "Understand TdhGetEventInformation field requirements")]
struct Args {
    /// Output directory for results
    #[arg(short, long, default_value = "output")]
    output: PathBuf,
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Args::parse();
    let output_dir = &args.output;

    if let Err(e) = fs::create_dir_all(output_dir) {
        eprintln!("Failed to create output directory: {}", e);
        std::process::exit(1);
    }

    let txt_path = output_dir.join("tdh_fabricate_output.txt");

    // Capture stdout by using a custom writer that tee's to both console and buffer
    let original_stdout = io::stdout();
    let captured = CapturedOutput::new(original_stdout);

    println!("============================================================");
    println!("  TDH Fabricate Experiment");
    println!("  Understanding TdhGetEventInformation field requirements");
    println!("============================================================");
    println!();
    println!("NOTE: Experiments 1, 3, 5, 6, 7 require administrator privileges");
    println!("      for kernel ETW tracing. They will be skipped if not admin.");
    println!("      Experiments 2 and 4 work without admin (pure fabrication).");
    println!();

    // These experiments work WITHOUT admin (pure fabrication, no ETW session)
    experiments::experiment_2_minimal_fabrication();
    experiments::experiment_4_version_probing();

    // These experiments need kernel tracing (admin required)
    // They gracefully skip if tracing fails
    experiments::experiment_1_baseline();
    experiments::experiment_3_field_sensitivity();
    experiments::experiment_5_modify_real_record();
    experiments::experiment_6_flags_and_properties();
    experiments::experiment_7_userdata_effects();

    println!();
    println!("============================================================");
    println!("  All experiments complete.");
    println!("============================================================");

    // Get captured output
    let txt_output = captured.get_output();

    // Write human-readable text output
    if let Err(e) = fs::write(&txt_path, &txt_output) {
        eprintln!("Failed to write text output: {}", e);
    } else {
        log::info!("Text output saved to {}", txt_path.display());
    }

    // Write JSON summary
    let json_path = output_dir.join("tdh_fabricate_output.json");
    let summary = serde_json::json!({
        "experiment": "tdh-fabricate",
        "description": "Understanding TdhGetEventInformation field requirements",
        "output_files": {
            "text": txt_path.display().to_string(),
            "json": json_path.display().to_string(),
        },
        "experiments": [
            "experiment_1_baseline",
            "experiment_2_minimal_fabrication",
            "experiment_3_field_sensitivity",
            "experiment_4_version_probing",
            "experiment_5_modify_real_record",
            "experiment_6_flags_and_properties",
            "experiment_7_userdata_effects",
        ],
        "notes": {
            "admin_required": "Experiments 1, 3, 5, 6, 7 require administrator privileges",
            "no_admin_needed": "Experiments 2 and 4 work without admin (pure fabrication)",
        },
    });

    match serde_json::to_string_pretty(&summary) {
        Ok(json) => {
            if let Err(e) = fs::write(&json_path, json) {
                eprintln!("Failed to write JSON output: {}", e);
            } else {
                log::info!("JSON output saved to {}", json_path.display());
            }
        }
        Err(e) => {
            eprintln!("Failed to serialize JSON: {}", e);
        }
    }
}

/// A writer that tees output to both the original writer and an internal buffer.
struct CapturedOutput<W: Write> {
    original: W,
    buffer: Vec<u8>,
}

impl<W: Write> CapturedOutput<W> {
    fn new(original: W) -> Self {
        Self {
            original,
            buffer: Vec::new(),
        }
    }

    fn get_output(&self) -> String {
        String::from_utf8_lossy(&self.buffer).to_string()
    }
}

impl<W: Write> Write for CapturedOutput<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        self.original.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.original.flush()
    }
}
