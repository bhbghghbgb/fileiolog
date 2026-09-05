use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
pub enum TraceMode {
    /// Kernel tracing with EnableFlags and optional PERFINFO_GROUPMASK
    Kernel,
    /// User-mode tracing with keyword filtering
    User,
}

#[derive(Debug, Clone, Parser)]
#[command(name = "tdh-enumerator")]
#[command(about = "Enumerate all possible information about ETW events")]
pub struct AppConfig {
    /// Trace mode: kernel or user
    #[arg(short, long, value_enum, default_value_t = TraceMode::Kernel)]
    pub mode: TraceMode,

    /// Provider GUID (e.g., "90cbdc39-4a3e-11d1-84f4-0000f80464e3" for FileIo)
    #[arg(short, long)]
    pub guid: String,

    /// Duration to capture events (seconds)
    #[arg(short, long, default_value_t = 5)]
    pub duration: u64,

    /// EnableFlags for kernel tracing (hex, e.g., "0x02000000")
    #[arg(long)]
    pub enable_flags: Option<String>,

    /// PERFINFO_GROUPMASK as 8 hex values separated by commas
    /// Example: "0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000,0x00000000"
    #[arg(long)]
    pub group_mask: Option<String>,

    /// Keyword for user tracing (hex, e.g., "0x0")
    #[arg(long)]
    pub keyword: Option<String>,

    /// Level for provider (0-255, default 0xFF for all)
    #[arg(long, default_value_t = 255)]
    pub level: u8,

    /// Output directory for results
    #[arg(short, long, default_value = "output")]
    pub output: std::path::PathBuf,

    /// Output file prefix (files will be .json and .txt)
    #[arg(long, default_value = "tdh_output")]
    pub output_prefix: String,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,

    /// Do NOT trigger file-ops-trigger during the session
    #[arg(long)]
    pub no_trigger: bool,
}

impl AppConfig {
    pub fn parse_enable_flags(&self) -> Option<u32> {
        self.enable_flags.as_ref().and_then(|s| {
            let s = s.trim_start_matches("0x").trim_start_matches("0X");
            u32::from_str_radix(s, 16).ok()
        })
    }

    pub fn parse_group_mask(&self) -> Option<[u32; 8]> {
        self.group_mask.as_ref().and_then(|s| {
            let parts: Vec<&str> = s.split(',').collect();
            if parts.len() != 8 {
                return None;
            }
            let mut mask = [0u32; 8];
            for (i, part) in parts.iter().enumerate() {
                let part = part.trim().trim_start_matches("0x").trim_start_matches("0X");
                mask[i] = u32::from_str_radix(part, 16).ok()?;
            }
            Some(mask)
        })
    }

    pub fn parse_keyword(&self) -> u64 {
        self.keyword
            .as_ref()
            .and_then(|s| {
                let s = s.trim_start_matches("0x").trim_start_matches("0X");
                u64::from_str_radix(s, 16).ok()
            })
            .unwrap_or(0)
    }

    pub fn parse_provider_guid(&self) -> Result<windows::core::GUID, String> {
        parse_guid(&self.guid)
    }
}

fn parse_guid(s: &str) -> Result<windows::core::GUID, String> {
    let s = s.trim();
    // Try format: "XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX"
    let s = s.trim_start_matches('{').trim_end_matches('}');
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 5 {
        return Err(format!("Invalid GUID format: {}", s));
    }

    let data1 = u32::from_str_radix(parts[0], 16).map_err(|e| format!("Invalid data1: {}", e))?;
    let data2 = u16::from_str_radix(parts[1], 16).map_err(|e| format!("Invalid data2: {}", e))?;
    let data3 = u16::from_str_radix(parts[2], 16).map_err(|e| format!("Invalid data3: {}", e))?;

    let data4_hex = format!("{}{}", parts[3], parts[4]);

    let mut data4 = [0u8; 8];
    for i in 0..8 {
        data4[i] = u8::from_str_radix(&data4_hex[i * 2..i * 2 + 2], 16)
            .map_err(|e| format!("Invalid data4[{}]: {}", i, e))?;
    }

    Ok(windows::core::GUID {
        data1,
        data2,
        data3,
        data4,
    })
}
