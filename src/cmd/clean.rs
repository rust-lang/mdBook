use super::command_prelude::*;
use crate::get_book_dir;
use anyhow::Context;
use anyhow::Result;
use mdbook_driver::MDBook;
use std::mem::take;
use std::path::PathBuf;
use std::{fmt, fs};

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("clean")
        .about("Deletes a built book")
        .arg_dest_dir()
        .arg_root_dir()
}

// Clean command implementation
pub fn execute(args: &ArgMatches) -> Result<()> {
    let book_dir = get_book_dir(args);
    let book = MDBook::load(book_dir)?;

    let dir_to_remove = match args.get_one::<PathBuf>("dest-dir") {
        Some(dest_dir) => dest_dir.into(),
        None => book.root.join(&book.config.build.build_dir),
    };

    let removed = Clean::new(&dir_to_remove)?;
    println!("{removed}");

    Ok(())
}

/// Formats a number of bytes into a human readable SI-prefixed size.
/// Returns a tuple of `(quantity, units)`.
pub fn human_readable_bytes(bytes: u64) -> (f32, &'static str) {
    static UNITS: [&str; 7] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];
    let bytes = bytes as f32;
    let i = ((bytes.log2() / 10.0) as usize).min(UNITS.len() - 1);
    (bytes / 1024_f32.powi(i as i32), UNITS[i])
}

#[derive(Debug)]
pub struct Clean {
    num_files_removed: u64,
    num_dirs_removed: u64,
    total_bytes_removed: u64,
}

impl Clean {
    fn new(dir: &PathBuf) -> Result<Clean> {
        let mut files = vec![dir.clone()];
        let mut children = Vec::new();
        let mut num_files_removed = 0;
        let mut num_dirs_removed = 0;
        let mut total_bytes_removed = 0;

        if dir.exists() {
            while !files.is_empty() {
                for file in files {
                    if let Ok(meta) = file.metadata() {
                        // Note: This can over-count bytes removed for hard-linked
                        // files. It also under-counts since it only counts the exact
                        // byte sizes and not the block sizes.
                        total_bytes_removed += meta.len();
                    }
                    if file.is_file() {
                        num_files_removed += 1;
                    } else if file.is_dir() {
                        num_dirs_removed += 1;
                        for entry in fs::read_dir(file)? {
                            children.push(entry?.path());
                        }
                    }
                }
                files = take(&mut children);
            }
            fs::remove_dir_all(&dir).with_context(|| "Unable to remove the build directory")?;
        }

        Ok(Clean {
            num_files_removed,
            num_dirs_removed,
            total_bytes_removed,
        })
    }
}

impl fmt::Display for Clean {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Removed ")?;
        match (self.num_files_removed, self.num_dirs_removed) {
            (0, 0) => write!(f, "0 files")?,
            (0, 1) => write!(f, "1 directory")?,
            (0, 2..) => write!(f, "{} directories", self.num_dirs_removed)?,
            (1, _) => write!(f, "1 file")?,
            (2.., _) => write!(f, "{} files", self.num_files_removed)?,
        }

        if self.total_bytes_removed == 0 {
            Ok(())
        } else {
            // Don't show a fractional number of bytes.
            if self.total_bytes_removed < 1024 {
                write!(f, ", {}B total", self.total_bytes_removed)
            } else {
                let (bytes, unit) = human_readable_bytes(self.total_bytes_removed);
                write!(f, ", {bytes:.2}{unit} total")
            }
        }
    }
}
