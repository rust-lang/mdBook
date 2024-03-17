use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

use resolve_path::PathResolveExt;

use super::command_prelude::*;
use mdbook::config::ShelfConfig;
use mdbook::errors::Result;
use mdbook::MDBook;

const SHELF_DIR: &str = "shelf";
const REPOS_DIR: &str = "repositories";
const INDEX_MD_FILE: &str = "index.md";
const INDEX_HTML_FILE: &str = "index.html";
const BOOKS_DIR: &str = "books";
const BOOKSHELF_DIR: &str = "bookshelf";
const SUMMARY_MD_FILE: &str = "SUMMARY.md";

pub fn make_subcommand() -> Command {
    Command::new("shelf").about("Build a bookshelf from shelf.toml file")
}

struct BookContext {
    title: String,
    desc: String,
    authors: String,
}

struct ShelfContext {
    book_dir: PathBuf,
    source_dir: PathBuf,
    url_prefix: String,
    url: String,
    index_file_name: PathBuf,
    summary_file_name: PathBuf,
}

fn update_index(
    index_file: &mut File,
    summary_file: &mut File,
    shelf_source: &PathBuf,
    root_prefix: &str,
    context: BookContext,
) -> Result<()> {
    // Create post in index file
    let book_link = format!(
        "## [{title}](<{prefix}/{BOOKSHELF_DIR}/{BOOKS_DIR}/{title}/{INDEX_HTML_FILE}>)",
        title = context.title,
        prefix = root_prefix
    );
    writeln!(index_file, "{book_link}")?;
    writeln!(index_file)?;
    writeln!(index_file, "{desc}", desc = context.desc)?;

    // Create a separate chapter file for the book
    let fixed_title = context.title.replace(' ', "_");
    let file_name = format!("{fixed_title}.md");
    let mut file_path = shelf_source.clone();
    file_path.push(&file_name);
    let mut bf = File::create(file_path)?;
    writeln!(bf, "{book_link}")?;
    writeln!(bf)?;
    writeln!(bf, "{desc}", desc = context.desc)?;
    writeln!(bf)?;
    writeln!(bf)?;
    writeln!(bf, "*{authors}*", authors = context.authors)?;

    // Add the chapter to the summary
    writeln!(
        summary_file,
        "- [{title}](./{file_name})",
        title = context.title
    )?;
    writeln!(summary_file)?;

    Ok(())
}

fn process_book(path: &str, books_dir: &PathBuf, shelf_url: &str) -> Result<BookContext> {
    let book_dir = path.try_resolve()?;
    let book_dir = std::fs::canonicalize(book_dir)?;
    let mut book = MDBook::load(book_dir)?;

    // Build book
    let title = book.config.book.title.clone().unwrap();
    let mut build_path = books_dir.to_owned();
    build_path.push(title);
    book.config.build.build_dir = build_path;
    // Create back reference to bookshelf
    book.config.book.shelf_url = Some(shelf_url.to_owned());
    book.build()?;

    let book_context = BookContext {
        title: book.config.book.title.unwrap_or_default(),
        desc: book.config.book.description.unwrap_or_default(),
        authors: book.config.book.authors.join(", "),
    };

    Ok(book_context)
}

fn setup_shelf_book(config: &ShelfConfig) -> Result<ShelfContext> {
    let book_dir = format!("{BOOKSHELF_DIR}/{SHELF_DIR}");
    let book = MDBook::init(&book_dir).build()?;
    let build_dir = book.config.build.build_dir.to_str().unwrap_or_default();
    let url_prefix = if !config.root_url_prefix.is_empty() {
        let mut full_prefix = "/".to_owned();
        full_prefix.push_str(&config.root_url_prefix);
        full_prefix
    } else {
        config.root_url_prefix.to_owned()
    };
    let url = format!("{url_prefix}/{book_dir}/{build_dir}/{INDEX_HTML_FILE}");

    let mut index_file_name = book.source_dir();
    index_file_name.push(INDEX_MD_FILE);

    let mut summary_file_name = book.source_dir();
    summary_file_name.push(SUMMARY_MD_FILE);

    Ok(ShelfContext {
        book_dir: book_dir.into(),
        source_dir: book.source_dir(),
        url_prefix,
        url,
        index_file_name,
        summary_file_name,
    })
}

pub fn execute(_args: &ArgMatches) -> Result<()> {
    let mut file = File::open("shelf.toml")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let shelf_config: ShelfConfig = toml::from_str(&contents)?;

    let _ = std::fs::remove_dir_all(BOOKSHELF_DIR);
    let _ = std::fs::remove_dir_all(REPOS_DIR);

    let shelf_context = setup_shelf_book(&shelf_config)?;

    let mut index_file = File::create(shelf_context.index_file_name).unwrap();
    writeln!(index_file, "# {title}", title = shelf_config.title)?;
    writeln!(index_file)?;

    let mut summary_file = File::create(shelf_context.summary_file_name).unwrap();
    writeln!(summary_file, "# Summary")?;
    writeln!(
        summary_file,
        "- [{title}](./{INDEX_MD_FILE})",
        title = shelf_config.title
    )?;

    let mut books_build_dir = std::env::current_dir()?;
    books_build_dir.push(BOOKSHELF_DIR);
    books_build_dir.push(BOOKS_DIR);

    for sb in &shelf_config.books {
        let book_path = if let Some(url) = &sb.git_url {
            prepare_git(sb, url)
        } else if let Some(path) = &sb.path {
            Some(path.to_owned())
        } else {
            warn!("Neither path or git specified. Invalid book");
            None
        };

        if let Some(path) = book_path {
            let update_context = process_book(&path, &books_build_dir, &shelf_context.url)?;
            let _ = update_index(
                &mut index_file,
                &mut summary_file,
                &shelf_context.source_dir,
                &shelf_context.url_prefix,
                update_context,
            )?;
        }
    }

    let shelf = MDBook::load(&shelf_context.book_dir)?;
    shelf.build()?;

    Ok(())
}

fn prepare_git(sb: &mdbook::config::ShelfBook, url: &String) -> Option<String> {
    println!("{:?}", sb);

    // Prepare checkout directory name
    let path = sb.path.clone().unwrap_or("root".to_owned());
    let repo_raw_name = url.split('/').last().unwrap_or(&path);
    let repo_name = format!("{repo_raw_name}-{path}");
    let mut checkout_path = PathBuf::from(REPOS_DIR);
    checkout_path.push(repo_name);

    let book_path = if let Some(path) = &sb.path {
        let mut bp = checkout_path.clone();
        bp.push(path);
        bp
    } else {
        checkout_path.clone()
    };

    let repo = match git2::Repository::open(&checkout_path) {
        Ok(repo) => repo,
        Err(_) => match git2::Repository::clone(&url, &checkout_path) {
            Ok(repo) => repo,
            Err(e) => panic!("failed to clone: {}", e),
        },
    };

    if let Some(refname) = &sb.git_ref {
        // branch or a tag (v0.1.1) or a commit (8e8128)
        let (object, reference) = if let Ok((object, reference)) = repo.revparse_ext(refname) {
            (object, reference)
        } else if let Ok((object, reference)) = repo.revparse_ext(&format!("origin/{refname}")) {
            (object, reference)
        } else {
            panic!("Could not checkout {refname}");
        };

        repo.checkout_tree(&object, None)
            .expect("Failed to checkout");

        match reference {
            // gref is an actual reference like branches or tags
            Some(gref) => repo.set_head(gref.name().unwrap()),
            // this is a commit, not a reference
            None => repo.set_head_detached(object.id()),
        }
        .expect("Failed to set HEAD");
    }

    Some(book_path.to_str().unwrap().to_owned())
}

#[test]
fn test_parse_toml() {
    let toml = r#"
root_url_prefix = "myprefix"

[[book]]
git_url = "firsturl"
git_ref = "shelf"
path = "guide"

[[book]]
git_url = "secondurl"

[[book]]
path = "../test_book"
"#;
    let cfg: ShelfConfig = toml::from_str(&toml).unwrap();
    assert_eq!(cfg.root_url_prefix, "myprefix");

    let book = &cfg.books[0];
    assert_eq!(book.git_url.clone().unwrap(), "firsturl");
    assert_eq!(book.git_ref.clone().unwrap(), "shelf");
    assert_eq!(book.path.clone().unwrap(), "guide");

    let book = &cfg.books[1];
    assert_eq!(book.git_url.clone().unwrap(), "secondurl");
    assert!(book.git_ref.is_none());
    assert!(book.path.is_none());

    let book = &cfg.books[2];
    assert_eq!(book.path.clone().unwrap(), "../test_book");
}

#[test]
fn test_config_defaults() {
    let toml = r#"
[[book]]
path = "../test_book"
    "#;
    let cfg: ShelfConfig = toml::from_str(&toml).unwrap();
    assert_eq!(cfg.root_url_prefix, "".to_owned());
    assert_eq!(cfg.title, "Bookshelf".to_owned());
}
