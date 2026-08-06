pub mod events;
pub mod file_ops;
pub mod fileio_events;
pub mod persist;
pub mod trace_session;

/// Build a PERFINFO_GROUPMASK from a combined mask value.
/// The mask value has the group index encoded in the high 3 bits.
pub fn build_group_mask(mask_value: u32) -> [u32; 8] {
    let mut masks = [0u32; 8];
    let group_index = ((mask_value >> 29) & 0x07) as usize;
    masks[group_index] = mask_value;
    masks
}
