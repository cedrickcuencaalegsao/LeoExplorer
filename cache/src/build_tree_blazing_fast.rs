use shared::FileEntry;
use std::path::PathBuf;
use std::collections::{HashMap, HashSet};

pub fn build_tree_blazing_fast(scanned_paths: &[PathBuf]) -> Vec<FileEntry> {
    shared::reset_tree_build_progress(scanned_paths.len() as u64);

    #[derive(Default)]
    struct Node {
        is_dir: bool,
        children: Vec<PathBuf>,
    }

    let mut nodes: HashMap<PathBuf, Node> = HashMap::with_capacity(scanned_paths.len());

    for path in scanned_paths {
        let is_dir = path.is_dir();

        nodes.entry(path.clone()).or_insert_with(|| Node {
            is_dir,
            children: Vec::new(),
        });

        if let Some(parent) = path.parent() {
            nodes
                .entry(parent.to_path_buf())
                .or_default()
                .children
                .push(path.clone());
        }
    }

    let has_parent: HashSet<&PathBuf> = nodes.values().flat_map(|n| n.children.iter()).collect();

    let mut roots: Vec<PathBuf> = nodes
        .keys()
        .filter(|p| !has_parent.contains(p))
        .cloned()
        .collect();

    roots.sort_unstable();

    fn build_entry(path: &PathBuf, nodes: &HashMap<PathBuf, Node>) -> FileEntry {
        let node = nodes.get(path);
        let is_dir = node.map(|n| n.is_dir).unwrap_or(false);

        // from_path now populates flags automatically — no extra work needed here.
        let mut entry = FileEntry::from_path(path, is_dir);
        shared::log_cache_tree_insert(&entry.path, entry.is_dir);

        entry.children = node.and_then(|n| {
            if n.children.is_empty() {
                None
            } else {
                const PAR_CHILD_THRESHOLD: usize = 16;
                let mut v: Vec<FileEntry> = if n.children.len() >= PAR_CHILD_THRESHOLD {
                    n.children
                        .iter()
                        .map(|c| build_entry(c, nodes))
                        .collect()
                } else {
                    n.children.iter().map(|c| build_entry(c, nodes)).collect()
                };

                v.sort_unstable_by(|a, b| match (a.is_dir, b.is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                });
                Some(v)
            }
        });

        entry
    }

    let mut result: Vec<FileEntry> = roots.into_iter().map(|r| build_entry(&r, &nodes)).collect();

    result.sort_unstable_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    result
}