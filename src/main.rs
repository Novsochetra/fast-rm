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

    // Single WalkDir pass
    let mut files: Vec<_> = Vec::new();
    let mut dirs: Vec<_> = Vec::new();

    for entry in WalkDir::new(&dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.file_name().map_or(false, |n| {
            let s = n.to_string_lossy();
            s.starts_with("._") || s == ".DS_Store"
        }) {
            continue;
        }

        if entry.file_type().is_file() {
            files.push(path.to_path_buf());
        } else if entry.file_type().is_dir() {
            dirs.push(path.to_path_buf());
        }
    }

    // Sort directories bottom-up
    dirs.sort_by_key(|d| std::cmp::Reverse(d.components().count()));

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
    files.par_iter().for_each(|path| {
        let _ = fs::remove_file(path);
        pb.inc(1);
    });

    // Delete directories in parallel
    dirs.par_iter().for_each(|path| {
        let _ = fs::remove_dir(path);
        pb.inc(1);
    });

    pb.finish_with_message("✅ Directory fully removed!");
}
