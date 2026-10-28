use rayon::prelude::*;
use shared::FileEntry;
use std::path::PathBuf;
use std::time::Instant;

const NONE: u32 = u32::MAX;
const PAR_CHILD_THRESHOLD: usize = 16;

/// Read-only, index-based tree shared across rayon workers.
struct Tree<'a> {
    /// Real (scanned) paths, sorted + deduped. Indices 0..sorted.len().
    sorted: Vec<&'a PathBuf>,
    /// Synthetic parents (parents of top-level scanned paths that were not
    /// themselves scanned). Index = sorted.len() + position here.
    synth: Vec<PathBuf>,
    /// CSR child lists: children of node i are kids[start[i]..start[i + 1]].
    start: Vec<u32>,
    kids: Vec<u32>,
    is_dir: Vec<bool>,
}

impl<'a> Tree<'a> {
    #[inline]
    fn path(&self, i: usize) -> &PathBuf {
        if i < self.sorted.len() {
            self.sorted[i]
        } else {
            &self.synth[i - self.sorted.len()]
        }
    }

    #[inline]
    fn children(&self, i: usize) -> &[u32] {
        &self.kids[self.start[i] as usize..self.start[i + 1] as usize]
    }
}

#[inline]
fn sort_entries(v: &mut [FileEntry]) {
    // Directories first, then case-insensitive name. The lowercase key is
    // computed once per element instead of on every comparison.
    v.sort_by_cached_key(|e| (!e.is_dir, e.name.to_lowercase()));
}

fn build_entry(i: usize, t: &Tree) -> FileEntry {
    let mut entry = FileEntry::from_path(t.path(i), t.is_dir[i]);
    shared::log_cache_tree_insert(&entry.path, entry.is_dir);

    let kids = t.children(i);
    log::trace!(
        "build_tree: entry {} [{}] children={}",
        t.path(i).display(),
        if t.is_dir[i] { "dir" } else { "file" },
        kids.len()
    );
    entry.children = if kids.is_empty() {
        None
    } else {
        let parallel = kids.len() >= PAR_CHILD_THRESHOLD;
        if parallel {
            // Only log the big fan-outs so the log stays readable.
            log::debug!(
                "Building {} children of {} in parallel",
                kids.len(),
                t.path(i).display()
            );
        }
        let mut v: Vec<FileEntry> = if parallel {
            kids.par_iter().map(|&c| build_entry(c as usize, t)).collect()
        } else {
            kids.iter().map(|&c| build_entry(c as usize, t)).collect()
        };
        sort_entries(&mut v);
        Some(v)
    };

    entry
}

pub fn build_tree_blazing_fast(scanned_paths: &[PathBuf]) -> Vec<FileEntry> {
    let total_start = Instant::now();
    log::info!(
        "build_tree: starting with {} scanned paths",
        scanned_paths.len()
    );
    shared::reset_tree_build_progress(scanned_paths.len() as u64);

    // 1. Sort + dedup references (no PathBuf clones, no hashing).
    //    Path ordering is component-wise, so every directory sorts directly
    //    before its descendants and a subtree is a contiguous run.
    let t = Instant::now();
    let mut sorted: Vec<&PathBuf> = scanned_paths.iter().collect();
    sorted.par_sort_unstable();
    let before_dedup = sorted.len();
    sorted.dedup();
    let real_count = sorted.len();
    log::info!(
        "build_tree: [1/5] sorted {} paths in {:.3}s ({} duplicates removed)",
        before_dedup,
        t.elapsed().as_secs_f64(),
        before_dedup - real_count
    );

    // 2. Find each node's parent in one linear pass using an ancestor stack.
    let t = Instant::now();
    let mut parent: Vec<u32> = vec![NONE; real_count];
    let mut synth: Vec<PathBuf> = Vec::new();
    let mut last_synth: Option<usize> = None; // node index of last synthetic parent
    let mut stack: Vec<usize> = Vec::new();
    let mut max_depth_seen = 0usize;

    for i in 0..real_count {
        let p = sorted[i];

        while let Some(&top) = stack.last() {
            if p.starts_with(sorted[top]) {
                break;
            }
            stack.pop();
        }

        let par = p.parent();
        match stack.last() {
            Some(&top) if par == Some(sorted[top].as_path()) => {
                parent[i] = top as u32;
            }
            _ => {
                // Parent is not among the scanned paths: it becomes a root
                // (synthetic) node, shared by all its scanned children.
                if let Some(pp) = par {
                    let reuse = last_synth
                        .filter(|&s| synth[s - real_count].as_path() == pp);
                    let s = match reuse {
                        Some(s) => s,
                        None => {
                            let s = real_count + synth.len();
                            log::debug!(
                                "build_tree: synthetic parent created: {}",
                                pp.display()
                            );
                            synth.push(pp.to_path_buf());
                            parent.push(NONE);
                            last_synth = Some(s);
                            s
                        }
                    };
                    parent[i] = s as u32;
                }
            }
        }

        stack.push(i);
        max_depth_seen = max_depth_seen.max(stack.len());
    }
    log::info!(
        "build_tree: [2/5] resolved parents in {:.3}s ({} synthetic parents, max nesting {})",
        t.elapsed().as_secs_f64(),
        synth.len(),
        max_depth_seen
    );

    // 3. Build CSR child lists (counting sort), no per-node Vec allocations.
    let t = Instant::now();
    let n = parent.len();
    let mut start = vec![0u32; n + 1];
    for &p in &parent {
        if p != NONE {
            start[p as usize + 1] += 1;
        }
    }
    for i in 0..n {
        start[i + 1] += start[i];
    }
    let mut fill = start.clone();
    let mut kids = vec![0u32; start[n] as usize];
    for (i, &p) in parent.iter().enumerate() {
        if p != NONE {
            let pos = &mut fill[p as usize];
            kids[*pos as usize] = i as u32;
            *pos += 1;
        }
    }
    drop(fill);

    let roots: Vec<usize> = (0..parent.len()).filter(|&i| parent[i] == NONE).collect();
    drop(parent);
    log::info!(
        "build_tree: [3/5] built child lists in {:.3}s ({} nodes, {} links, {} roots)",
        t.elapsed().as_secs_f64(),
        n,
        kids.len(),
        roots.len()
    );

    // 4. is_dir: anything with children is a directory (no syscall). Only
    //    childless real paths need a stat, and those run in parallel.
    let t = Instant::now();
    let stat_needed = (0..real_count)
        .filter(|&i| start[i + 1] == start[i])
        .count();
    let is_dir: Vec<bool> = (0..n)
        .into_par_iter()
        .map(|i| {
            if start[i + 1] > start[i] || i >= real_count {
                true
            } else {
                sorted[i].is_dir()
            }
        })
        .collect();
    let dir_count = is_dir.iter().filter(|&&d| d).count();
    log::info!(
        "build_tree: [4/5] resolved is_dir in {:.3}s ({} stat calls skipped, {} performed; {} dirs, {} files)",
        t.elapsed().as_secs_f64(),
        n - stat_needed,
        stat_needed,
        dir_count,
        n - dir_count
    );

    let tree = Tree {
        sorted,
        synth,
        start,
        kids,
        is_dir,
    };

    // 5. Build FileEntry values in parallel (order preserved by indexed collect).
    let t = Instant::now();
    let mut result: Vec<FileEntry> = roots
        .par_iter()
        .map(|&r| build_entry(r, &tree))
        .collect();

    sort_entries(&mut result);
    log::info!(
        "build_tree: [5/5] built {} FileEntry nodes in {:.3}s",
        n,
        t.elapsed().as_secs_f64()
    );

    log::info!(
        "build_tree: done in {:.3}s ({} root entries)",
        total_start.elapsed().as_secs_f64(),
        result.len()
    );

    result
}