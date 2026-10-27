pub fn cache_build_scan_percent(completed_roots: usize, total_roots: u32) -> u8 {
    let t = total_roots.max(1) as u64;
    let c = completed_roots as u64;
    if c >= t {
        55
    } else {
        u8::try_from((c * 55) / t).unwrap_or(0).min(54)
    }
}