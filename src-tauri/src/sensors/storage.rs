//! Largest installed applications by on-disk size. A full-disk "largest
//! files" walk would take minutes and hammer I/O, so the scan is scoped to
//! the platform's applications directory and runs on a slow background
//! cadence, never on the metrics tick.

use crate::state::ProcessEntry;
use std::fs;
use std::path::Path;

#[cfg(target_os = "macos")]
const APP_DIRS: &[&str] = &["/Applications", "/Applications/Utilities"];
#[cfg(windows)]
const APP_DIRS: &[&str] = &["C:\\Program Files", "C:\\Program Files (x86)"];
#[cfg(not(any(target_os = "macos", windows)))]
const APP_DIRS: &[&str] = &["/usr/share/applications"];

pub fn top_apps(limit: usize) -> Vec<ProcessEntry> {
    let mut apps: Vec<ProcessEntry> = Vec::new();

    for dir in APP_DIRS {
        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            #[cfg(target_os = "macos")]
            let is_app = path.extension().is_some_and(|e| e == "app");
            #[cfg(not(target_os = "macos"))]
            let is_app = path.is_dir();
            if !is_app {
                continue;
            }
            let Some(name) = path.file_stem().map(|s| s.to_string_lossy().into_owned()) else {
                continue;
            };
            apps.push(ProcessEntry {
                name,
                cpu_percent: 0.0,
                mem_bytes: dir_size(&path),
            });
        }
    }

    apps.sort_by(|a, b| b.mem_bytes.cmp(&a.mem_bytes));
    apps.truncate(limit);
    apps
}

/// Recursive logical size; ignores unreadable entries, never follows symlinks.
fn dir_size(path: &Path) -> u64 {
    let Ok(meta) = path.symlink_metadata() else {
        return 0;
    };
    if meta.is_symlink() {
        return 0;
    }
    if meta.is_file() {
        return meta.len();
    }
    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };
    entries
        .flatten()
        .map(|e| dir_size(&e.path()))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_largest_apps() {
        let apps = top_apps(3);
        assert!(!apps.is_empty(), "expected at least one installed app");
        assert!(apps[0].mem_bytes > 0, "largest app reports a size");
        assert!(
            apps.windows(2).all(|w| w[0].mem_bytes >= w[1].mem_bytes),
            "sorted descending"
        );
    }
}
