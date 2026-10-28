#[derive(Debug, Clone, Copy, Default)]
pub struct DirectoryStatsTraversalState {
    pub visited_entries: usize,
    pub limit_hit: bool,
}
