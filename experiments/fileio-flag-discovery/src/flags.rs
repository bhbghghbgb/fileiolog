use crate::trace_session::TraceConfig;

pub(crate) struct Flag {
    pub name: String,
    pub enable_flags: Option<u32>,
    pub group_mask: Option<u32>,
}

pub(crate) fn build_all_flags() -> Vec<Flag> {
    vec![
        Flag { name: "EF:DISK_FILE_IO".into(), enable_flags: Some(0x00000200), group_mask: None },
        Flag { name: "EF:FILE_IO".into(), enable_flags: Some(0x02000000), group_mask: None },
        Flag { name: "EF:FILE_IO_INIT".into(), enable_flags: Some(0x04000000), group_mask: None },
        Flag { name: "EF:VAMAP".into(), enable_flags: Some(0x00008000), group_mask: None },
        Flag { name: "GM:PERF_FILENAME".into(), enable_flags: None, group_mask: Some(0x00000200) },
        Flag { name: "GM:PERF_FILE_IO".into(), enable_flags: None, group_mask: Some(0x02000000) },
        Flag { name: "GM:PERF_FILE_IO_INIT".into(), enable_flags: None, group_mask: Some(0x04000000) },
        Flag { name: "GM:PERF_VAMAP".into(), enable_flags: None, group_mask: Some(0x00008000) },
        Flag { name: "GM:PERF_FLT_IO_INIT".into(), enable_flags: None, group_mask: Some(0x80080000) },
        Flag { name: "GM:PERF_FLT_IO".into(), enable_flags: None, group_mask: Some(0x80100000) },
        Flag { name: "GM:PERF_FLT_FASTIO".into(), enable_flags: None, group_mask: Some(0x80200000) },
        Flag { name: "GM:PERF_FLT_IO_FAILURE".into(), enable_flags: None, group_mask: Some(0x80400000) },
    ]
}

pub(crate) fn merge_flags(flags: &[Flag], indices: &[usize]) -> TraceConfig {
    let mut enable_flags = 0u32;
    let mut group_mask = 0u32;

    for &idx in indices {
        if let Some(ef) = flags[idx].enable_flags { enable_flags |= ef; }
        if let Some(gm) = flags[idx].group_mask { group_mask |= gm; }
    }

    let session_name = format!(
        "FlagDiscovery-{}",
        indices.iter()
            .map(|i| flags[*i].name.replace(":", "_").replace("+", "_"))
            .collect::<Vec<_>>()
            .join("__")
    );

    TraceConfig {
        session_name,
        enable_flags: if enable_flags != 0 { Some(enable_flags) } else { None },
        group_mask: if group_mask != 0 { Some(to_group_mask(group_mask)) } else { None },
    }
}

pub(crate) fn combo_name(flags: &[Flag], indices: &[usize]) -> String {
    indices.iter().map(|i| flags[*i].name.as_str()).collect::<Vec<_>>().join(" + ")
}

fn to_group_mask(mask_value: u32) -> [u32; 8] {
    let mut masks = [0u32; 8];
    masks[((mask_value >> 29) & 0x07) as usize] = mask_value;
    masks
}
