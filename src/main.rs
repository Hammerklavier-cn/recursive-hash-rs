use clap::Parser;
use hasher::Hasher;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

mod cli;
mod finder;
mod hasher;

fn main() {
    let args = cli::Args::parse();
    if std::env::var("RUST_LOG").is_err() {
        if args.verbose {
            unsafe {
                std::env::set_var("RUST_LOG", "debug");
            }
        } else {
            unsafe {
                std::env::set_var("RUST_LOG", "info");
            }
        }
    }

    env_logger::init();

    match args.mode {
        cli::Mode::Cli {
            paths,
            excludes,
            hasher: hasher_mode,
            out,
            audit,
        } => {
            log::debug!(
                "Paths: {:?}, Excludes: {:?}, Hasher: {:?}, Output: {}, Audit: {:?}",
                paths,
                excludes,
                hasher_mode,
                out,
                audit
            );

            if let Some(audit_path) = audit {
                // Audit mode: verify files against checklist
                audit_files(&audit_path, &paths, &excludes, &hasher_mode);
            } else {
                // Generate mode: create checksums for files
                generate_checksums(&paths, &excludes, &hasher_mode, &out);
            }
        }
        cli::Mode::Gui => {
            log::error!("GUI mode not yet implemented");
            println!("GUI mode is not yet implemented. Please use CLI mode instead.");
        }
    }
}

/// Generate checksums for files and write to output file
fn generate_checksums(
    paths: &[PathBuf],
    excludes: &[PathBuf],
    hasher_mode: &cli::HasherMode,
    out: &str,
) {
    let mut success_count = 0;
    let mut error_count = 0;

    // Create output file
    let output_file = match File::create(out) {
        Ok(f) => f,
        Err(e) => {
            log::error!("Failed to create output file '{}': {}", out, e);
            std::process::exit(1);
        }
    };
    let mut writer = BufWriter::new(output_file);

    // Find all files to hash
    let files = finder::find_files(paths.to_vec(), excludes.to_vec());
    log::info!("Found {} files to hash", files.len());

    for file_path in &files {
        match File::open(file_path) {
            Ok(mut file) => {
                let hash = match hasher_mode {
                    cli::HasherMode::Md5 => hasher::Md5Hasher.get_hash(&mut file),
                    cli::HasherMode::Sha1 => hasher::Sha1Hasher.get_hash(&mut file),
                    cli::HasherMode::Sha256 => hasher::Sha256Hasher.get_hash(&mut file),
                    cli::HasherMode::Sha384 => hasher::Sha384Hasher.get_hash(&mut file),
                    cli::HasherMode::Sha512 => hasher::Sha512Hasher.get_hash(&mut file),
                };
                let relative_path = file_path
                    .strip_prefix(".")
                    .unwrap_or(file_path)
                    .to_string_lossy();
                let line = format!("{}  {}\n", hash, relative_path);
                if let Err(e) = writer.write_all(line.as_bytes()) {
                    log::error!("Failed to write to output file: {}", e);
                    error_count += 1;
                } else {
                    success_count += 1;
                    log::debug!("Hashed: {} -> {}", relative_path, hash);
                }
            }
            Err(e) => {
                log::error!("Failed to open file '{:?}': {}", file_path, e);
                error_count += 1;
            }
        }
    }

    if let Err(e) = writer.flush() {
        log::error!("Failed to flush output file: {}", e);
    }

    log::info!(
        "Completed: {} files hashed successfully, {} errors",
        success_count,
        error_count
    );
}

/// Audit files against a checklist file
fn audit_files(
    checklist_path: &PathBuf,
    paths: &[PathBuf],
    excludes: &[PathBuf],
    hasher_mode: &cli::HasherMode,
) {
    // Read the checklist file
    let checklist_file = match File::open(checklist_path) {
        Ok(f) => f,
        Err(e) => {
            log::error!(
                "Failed to open checklist file '{:?}': {}",
                checklist_path,
                e
            );
            println!("ERROR: Cannot open checklist file: {}", e);
            return;
        }
    };

    let reader = BufReader::new(checklist_file);
    let mut total_count = 0;
    let mut pass_count = 0;
    let mut fail_count = 0;
    let mut missing_count = 0;

    // Find all files for reference
    let files = finder::find_files(paths.to_vec(), excludes.to_vec());
    let files_set: std::collections::HashSet<PathBuf> = files.into_iter().collect();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                log::error!("Failed to read checklist line: {}", e);
                continue;
            }
        };

        // Skip empty lines and comments
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse line: "hash  filename" or "hash filename"
        let parts: Vec<&str> = line.splitn(2, "  ").collect();
        let parts: Vec<&str> = if parts.len() == 2 {
            parts
        } else {
            // Try single space separator
            line.splitn(2, ' ').collect()
        };

        if parts.len() != 2 {
            log::warn!("Invalid checklist line: {}", line);
            continue;
        }

        let expected_hash = parts[0].trim().to_lowercase();
        let file_path_str = parts[1].trim();
        let file_path = PathBuf::from(file_path_str);

        total_count += 1;

        // Check if file exists in the scanned paths
        if !files_set.contains(&file_path) && !file_path.exists() {
            log::warn!("File not found: {}", file_path_str);
            println!("MISSING: {}", file_path_str);
            missing_count += 1;
            continue;
        }

        // Calculate actual hash
        match File::open(&file_path) {
            Ok(mut file) => {
                let actual_hash = match hasher_mode {
                    cli::HasherMode::Md5 => hasher::Md5Hasher.get_hash(&mut file),
                    cli::HasherMode::Sha1 => hasher::Sha1Hasher.get_hash(&mut file),
                    cli::HasherMode::Sha256 => hasher::Sha256Hasher.get_hash(&mut file),
                    cli::HasherMode::Sha384 => hasher::Sha384Hasher.get_hash(&mut file),
                    cli::HasherMode::Sha512 => hasher::Sha512Hasher.get_hash(&mut file),
                };

                if actual_hash == expected_hash {
                    log::debug!("PASS: {}", file_path_str);
                    pass_count += 1;
                } else {
                    log::warn!(
                        "FAIL: {} - expected {}, got {}",
                        file_path_str,
                        expected_hash,
                        actual_hash
                    );
                    println!("FAIL: {}", file_path_str);
                    fail_count += 1;
                }
            }
            Err(e) => {
                log::error!("Failed to open file '{:?}': {}", file_path, e);
                println!("ERROR: {} - {}", file_path_str, e);
                missing_count += 1;
            }
        }
    }

    log::info!(
        "Audit complete: {} passed, {} failed, {} missing out of {} total",
        pass_count,
        fail_count,
        missing_count,
        total_count
    );

    if fail_count > 0 || missing_count > 0 {
        std::process::exit(1);
    }
}
