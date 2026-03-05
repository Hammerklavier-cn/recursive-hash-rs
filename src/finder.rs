use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Find files recursively in directories (and files), excluding certain paths.
pub fn find_files(include: Vec<impl AsRef<Path>>, exclude: Vec<impl AsRef<Path>>) -> Vec<PathBuf> {
    let exclude_paths: Vec<PathBuf> = exclude
        .into_iter()
        .map(|p| p.as_ref().to_path_buf())
        .collect();

    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for path in include {
        let path = path.as_ref();
        if path.is_file() {
            if !is_excluded(path, &exclude_paths) {
                let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
                if seen.insert(canonical.clone()) {
                    result.push(canonical);
                }
            }
        } else if path.is_dir() {
            let files = find_files_in_dir(path, exclude_paths.clone());
            for file in files {
                let canonical = file.canonicalize().unwrap_or_else(|_| file.clone());
                if seen.insert(canonical.clone()) {
                    result.push(canonical);
                }
            }
        }
    }

    result
}

/// Find files recursively in certain directory.
pub fn find_files_in_dir(dir: impl AsRef<Path>, exclude: Vec<impl AsRef<Path>>) -> Vec<PathBuf> {
    let mut result = Vec::new();
    let exclude_paths: Vec<PathBuf> = exclude
        .into_iter()
        .map(|p| p.as_ref().to_path_buf())
        .collect();

    fn walk_dir(dir: &Path, exclude_paths: &[PathBuf], result: &mut Vec<PathBuf>) {
        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("Warning: Failed to read directory {:?}: {}", dir, e);
                return;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("Warning: Failed to read entry: {}", e);
                    continue;
                }
            };

            let path = entry.path();

            if is_excluded(&path, exclude_paths) {
                continue;
            }

            if path.is_file() {
                result.push(path);
            } else if path.is_dir() {
                walk_dir(&path, exclude_paths, result);
            }
        }
    }

    walk_dir(dir.as_ref(), &exclude_paths, &mut result);
    result
}

/// Check if a path should be excluded.
fn is_excluded(path: &Path, exclude_paths: &[PathBuf]) -> bool {
    // Get canonical (absolute) path for the file being checked (it must exist)
    let canonical_path = match path.canonicalize() {
        Ok(p) => p,
        Err(_) => path.to_path_buf(),
    };

    exclude_paths.iter().any(|exclude| {
        // Try to get canonical path for exclude pattern
        match exclude.canonicalize() {
            Ok(canonical_exclude) => {
                // Compare canonical paths
                canonical_path.starts_with(&canonical_exclude)
                    || canonical_path == canonical_exclude
            }
            Err(_) => {
                // If exclude path doesn't exist, fall back to string comparison
                // Normalize both paths by removing `.` components
                let normalized_path = normalize_path(&canonical_path);
                let normalized_exclude = normalize_path(exclude);
                normalized_path.starts_with(&normalized_exclude)
                    || normalized_path == normalized_exclude
            }
        }
    })
}

/// Normalize a path by removing `.` components.
fn normalize_path(path: &Path) -> PathBuf {
    let mut components = path.components().peekable();
    let mut result = PathBuf::new();

    while let Some(component) = components.next() {
        match component {
            std::path::Component::CurDir => {
                // Skip `.` components
                if result.components().count() == 0 {
                    // Check if next component exists - if not, this is just `.`
                    if components.peek().is_none() {
                        result.push(".");
                    }
                }
            }
            _ => result.push(component),
        }
    }

    result
}
