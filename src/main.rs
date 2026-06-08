use std::io::Write;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use workspace::{apply_patch, compute_patch, show_diff};

#[derive(Parser)]
#[command(
    name = "diff-tgz",
    about = "Compute and apply VCDIFF binary patches between tgz archives"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Show a human-readable unified diff between two tgz archives (for use as
    /// a git difftool / external diff driver).
    ///
    /// Exit code: 0 = identical, 1 = differ.
    ///
    /// git difftool:    diff-tgz show "$LOCAL" "$REMOTE"
    /// External driver: diff-tgz show "$2" "$5"  (old-file new-file from 7-arg form)
    Show {
        /// The original (source) tgz archive.
        old: PathBuf,
        /// The updated (target) tgz archive.
        new: PathBuf,
    },
    /// Compute a VCDIFF patch between two tgz files and write it to stdout.
    ///
    /// Example: diff-tgz diff old.tgz new.tgz > diff.patch
    Diff {
        /// The original (source) tgz archive.
        old: PathBuf,
        /// The updated (target) tgz archive.
        new: PathBuf,
    },
    /// Apply a VCDIFF patch to a tgz file and write the result to stdout.
    ///
    /// Example: diff-tgz apply old.tgz diff.patch > new.tgz
    Apply {
        /// The original (source) tgz archive.
        old: PathBuf,
        /// The patch file produced by the `diff` subcommand.
        patch: PathBuf,
    },
}

fn read_file(path: &PathBuf) -> Vec<u8> {
    std::fs::read(path).unwrap_or_else(|e| {
        eprintln!("Error reading '{}': {}", path.display(), e);
        std::process::exit(1);
    })
}

fn write_stdout(data: &[u8]) {
    std::io::stdout()
        .lock()
        .write_all(data)
        .unwrap_or_else(|e| {
            eprintln!("Error writing to stdout: {}", e);
            std::process::exit(1);
        });
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Show { old, new } => {
            let old_bytes = read_file(&old);
            let new_bytes = read_file(&new);
            let (output, differs) = show_diff(&old_bytes, &new_bytes).unwrap_or_else(|e| {
                eprintln!("Error: {}", e);
                std::process::exit(2);
            });
            print!("{}", output);
            // Exit 1 when archives differ (mirrors `diff` convention).
            std::process::exit(if differs { 1 } else { 0 });
        }
        Command::Diff { old, new } => {
            let source = read_file(&old);
            let target = read_file(&new);
            let delta = compute_patch(&source, &target).unwrap_or_else(|e| {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            });
            write_stdout(&delta);
        }
        Command::Apply { old, patch } => {
            let source = read_file(&old);
            let delta = read_file(&patch);
            let restored = apply_patch(&source, &delta).unwrap_or_else(|e| {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            });
            write_stdout(&restored);
        }
    }
}
