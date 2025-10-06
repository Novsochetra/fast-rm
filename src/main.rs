use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Fast parallel rm -rf with progress for files & folders
#[derive(Parser, Debug)]
#[command(author, version, about = "Fast parallel rm -rf with progress")]
struct Args {
    /// Directory to remove
    dir: String,
}

fn main() {
    let args = Args::parse();
    let dir = args.dir;

    if !Path::new(&dir).exists() {
        eprintln!("Error: Directory '{}' does not exist!", dir);
        std::process::exit(1);
    }

    println!("Removing directory '{}'", dir);

    let entries: Vec<_> = WalkDir::new(&dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .collect();

    // Split into files and directories
    let mut files = Vec::new();
    let mut dirs = Vec::new();

    for entry in &entries {
        let path = entry.path();

        if let Ok(meta) = fs::symlink_metadata(path) {
            if meta.file_type().is_symlink() || meta.file_type().is_file() {
                files.push(path.to_path_buf());
            } else if meta.file_type().is_dir() {
                dirs.push(path.to_path_buf());
            }
        }
    }

    // Progress bar
    let total_items = files.len() + dirs.len();
    let pb = ProgressBar::new(total_items as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
        )
        .unwrap()
        .progress_chars("#>-"),
    );

    // Remove files and symlinks in parallel
    files.par_iter().for_each(|path| {
        if let Err(err) = fs::remove_file(path) {
            if err.kind() != std::io::ErrorKind::NotFound {
                eprintln!("Failed to remove file/symlink {}: {}", path.display(), err);
            }
        }
        pb.inc(1);
    });

    // Remove dirs (bottom-up)
    dirs.sort_by_key(|d| std::cmp::Reverse(d.components().count()));

    for dir_path in dirs {
        if let Ok(meta) = fs::symlink_metadata(&dir_path) {
            if meta.file_type().is_symlink() {
                // Just unlink symlink dir
                let _ = fs::remove_file(&dir_path);
            } else {
                if let Err(err) = fs::remove_dir(&dir_path) {
                    if err.kind() != std::io::ErrorKind::NotFound {
                        eprintln!("Failed to remove directory {}: {}", dir_path.display(), err);
                    }
                }
            }
        }
        pb.inc(1);
    }

    pb.finish_with_message("✅ Directory fully removed!");
}
