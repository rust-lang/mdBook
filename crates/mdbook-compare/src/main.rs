//! Utility to compare the output of two different versions of mdbook.

use std::path::Path;
use std::process::Command;

macro_rules! error {
    ($msg:literal $($arg:tt)*) => {
        eprint!("error: ");
        eprintln!($msg $($arg)*);
        std::process::exit(1);
    };
}

fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(mdbook1), Some(book1), Some(mdbook2), Some(book2)) =
        (args.next(), args.next(), args.next(), args.next())
    else {
        eprintln!("error: Expected four arguments: <exe1> <dir1> <exe2> <dir2>");
        std::process::exit(1);
    };
    let mdbook1 = Path::new(&mdbook1);
    let mdbook2 = Path::new(&mdbook2);
    let book1 = Path::new(&book1);
    let book2 = Path::new(&book2);
    let compare1 = Path::new("compare1");
    let compare2 = Path::new("compare2");
    clean(compare1);
    clean(compare2);
    clean(&book1.join("book"));
    clean(&book2.join("book"));
    build(mdbook1, book1);
    std::fs::rename(book1.join("book"), compare1).unwrap();
    build(mdbook2, book2);
    std::fs::rename(book2.join("book"), compare2).unwrap();
    diff(compare1, compare2);
}

fn clean(path: &Path) {
    if path.exists() {
        println!("removing {path:?}");
        std::fs::remove_dir_all(path).unwrap();
    }
}

fn build(mdbook: &Path, book: &Path) {
    println!("running `{mdbook:?} build` in `{book:?}`");
    let status = Command::new(mdbook)
        .arg("build")
        .current_dir(book)
        .status()
        .unwrap_or_else(|e| {
            error!("expected {mdbook:?} executable to exist: {e}");
        });
    if !status.success() {
        error!("process {mdbook:?} failed");
    }
    process(&book.join("book"));
}

fn process(path: &Path) {
    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            process(&path);
        } else {
            if path.extension().is_some_and(|ext| ext == "html") {
                tidy(&path);
                process_html(&path);
            } else {
                std::fs::remove_file(path).unwrap();
            }
        }
    }
}

fn process_html(path: &Path) {
    let content = std::fs::read_to_string(path).unwrap();
    let Some(start_index) = content.find("<main>") else {
        return;
    };
    let end_index = content.rfind("</main>").unwrap();
    let new_content = &content[start_index..end_index + 8];
    std::fs::write(path, new_content).unwrap();
}

fn tidy(path: &Path) {
    // quiet, no wrap, modify in place
    let args = "-q -w 0 -m --custom-tags yes --drop-empty-elements no";
    println!("running `tidy {args}` in `{path:?}`");
    let status = Command::new("tidy")
        .args(args.split(' '))
        .arg(path)
        .status()
        .expect("tidy should be installed");
    if !status.success() {
        // Exit code 1 is a warning.
        if status.code() != Some(1) {
            error!("tidy failed: {status}");
        }
    }
}

fn diff(a: &Path, b: &Path) {
    let args = "diff --no-index";
    println!("running `git {args} {a:?} {b:?}`");
    Command::new("git")
        .args(args.split(' '))
        .args([a, b])
        .status()
        .unwrap();
}
