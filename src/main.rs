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

    // Collect all files
    let files: Vec<_> = WalkDir::new(&dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .collect();

    // Collect all directories (for bottom-up removal)
    let dirs: Vec<_> = WalkDir::new(&dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .collect();

    let total_items = files.len() + dirs.len();
    let pb = ProgressBar::new(total_items as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
        )
        .unwrap()
        .progress_chars("#>-"),
    );

    // Delete files in parallel
    files.par_iter().for_each(|entry| {
        let path = entry.path();

        if let Some(name) = path.file_name() {
            let name_str = name.to_string_lossy();
            if name_str.starts_with("._") || name_str == ".DS_Store" {
                pb.inc(1);
                return;
            }
        }

        if let Err(err) = fs::remove_file(path) {
            if err.kind() != std::io::ErrorKind::NotFound {
                eprintln!("Failed to remove file {}: {}", path.display(), err);
            }
        }

        pb.inc(1);
    });

    // Remove directories bottom-up
    for dir_entry in dirs.iter().rev() {
        let path = dir_entry.path();

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    let _ = fs::remove_file(&p);
                } else if p.is_dir() {
                    let _ = fs::remove_dir_all(&p);
                }
            }
        }

        if let Err(err) = fs::remove_dir(path) {
            if err.kind() != std::io::ErrorKind::NotFound {
                eprintln!("Failed to remove directory {}: {}", path.display(), err);
            }
        }

        pb.inc(1); // Increment progress for the directory
    }

    pb.finish_with_message("✅ Directory fully removed!");
}
