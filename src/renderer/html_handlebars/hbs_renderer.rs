use renderer::html_handlebars::helpers;
use renderer::Renderer;
use book::{MDBook, Book};
use book::chapter::Chapter;
use book::toc::{TocItem, TocContent};
use utils;
use FILES;

use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use std::fs::{self, File};
use std::error::Error;
use std::io::{self, Read, Write};
use std::collections::BTreeMap;

use handlebars::Handlebars;

use serde_json;
use serde_json::value::ToJson;

pub struct HtmlHandlebars;

impl HtmlHandlebars {
    pub fn new() -> Self {
        HtmlHandlebars
    }
}

impl Renderer for HtmlHandlebars {

    /// Prepares the project and calls `render()`.
    fn build(&self, project_root: &PathBuf) -> Result<(), Box<Error>> {
        debug!("[fn]: build");

        let mut book_project = MDBook::new(&project_root);

        book_project.read_config();
        book_project.parse_books();
        book_project.link_translations();

        // Clean output directory

        // FIXME don't remove dotfiles such as .git/ folder. It's a common
        // practice to track gh-pages in a versioned output folder.

        //try!(utils::fs::remove_dir_content(&book_project.get_dest_base()));

        try!(self.render(&book_project));

        Ok(())
    }

    /// Renders the chapters and copies static assets.
    fn render(&self, book_project: &MDBook) -> Result<(), Box<Error>> {

        debug!("[*]: Check if book's base output folder exists");
        if let Err(_) = fs::create_dir_all(&book_project.get_dest_base()) {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Unexpected error when constructing path")
            ));
        }

        // TODO write print version html

        // Copy book's static assets

        if book_project.get_project_root().join("assets").exists() {

            let a = book_project.get_project_root().join("assets");
            let base = a.to_str().unwrap();

            let b = a.join("**").join("*");
            let include_glob = b.to_str().unwrap();

            let c = a.join("_*");
            let exclude_glob = c.to_str().unwrap();

            // anyone wants to catch errors?
            utils::fs::copy_files(include_glob,
                                  base,
                                  vec![exclude_glob],
                                  &book_project.get_dest_base());
        }

        // Copy template's static assets

        // If there is a template dir in the books's project folder, copy asset
        // files from there, otherwise copy from embedded assets.

        if book_project.get_template_dir().exists() {

            let a = book_project.get_template_dir();
            let base = a.to_str().unwrap();

            let b = a.join("**").join("*");
            let include_glob = b.to_str().unwrap();

            let c = a.join("_*");
            let exclude_glob = c.to_str().unwrap();

            // don't try!(), copy_files() will send error values when trying to copy folders that are part of the file glob
            //
            // Error {
            //     repr: Custom(
            //         Custom {
            //             kind: Other,
            //             error: StringError(
            //                 "Err(Error { repr: Custom(Custom { kind: InvalidInput, error: StringError(\"the source path is not an existing regular file\") }) })\n"
            //             )
            //         }
            //     )
            // }

            // anyone wants to catch errors?
            utils::fs::copy_files(include_glob,
                                  base,
                                  vec![exclude_glob],
                                  &book_project.get_dest_base());

        } else {
            try!(utils::fs::copy_data("data/_html-template/**/*",
                                      "data/_html-template/",
                                      vec!["data/_html-template/_*"],
                                      &book_project.get_dest_base()));
        }

        // Concatenate the content (as rendered from Markdown) of each chapter
        // for writing print.html in the end
        let mut print_content: String = String::new();

        debug!("[fn]: render");
        let mut handlebars = Handlebars::new();

        // Render the chapters of each book
        for (key, book) in &book_project.translations {

            // Look for the page template in these paths
            let mut tmpl_paths: Vec<PathBuf> = vec![];

            // default scheme: assets/_html-template/_layouts/page.hbs
            tmpl_paths.push(book_project.get_template_dir().join("_layouts").join("page.hbs"));
            // maybe the user doesn't use _layouts folder: assets/_html-template/page.hbs
            tmpl_paths.push(book_project.get_template_dir().join("page.hbs"));
            // also look for index.hbs which was the template name in v0.0.15
            tmpl_paths.push(book_project.get_template_dir().join("index.hbs"));

            let first_path_that_exists = |paths: &Vec<PathBuf>| -> Option<PathBuf> {
                for p in paths.iter() {
                    if p.exists() {
                        return Some(PathBuf::from(p));
                    }
                }
                None
            };

            let s = if let Some(p) = first_path_that_exists(&tmpl_paths) {
                try!(utils::fs::file_to_string(&p))
            } else {
                try!(utils::fs::get_data_file("data/_html-template/_layouts/page.hbs"))
            };

            // Register template
            debug!("[*]: Register handlebars template");
            try!(handlebars.register_template_string("page", s));

            // Register helpers
            debug!("[*]: Register handlebars helpers");
            handlebars.register_helper("toc", Box::new(helpers::toc::RenderToc));
            handlebars.register_helper("previous", Box::new(helpers::navigation::previous));
            handlebars.register_helper("next", Box::new(helpers::navigation::next));
            handlebars.register_helper("translations", Box::new(helpers::translations::TranslationsHelper));

            // Check if book's dest directory exists

            // If this is a single book, config.dest default is
            // `project_root/book`, and the earlier check will cover this.

            // If this is multi-language book, config.dest will
            // `project_book/book/key`, and so we check here for each book.

            debug!("[*]: Check if book's destination directory exists");
            if let Err(_) = fs::create_dir_all(book.config.get_dest()) {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    "Unexpected error when constructing destination path")
                ));
            }

            // If this is the main book of a multi-language book, add an
            // index.html to the project dest folder

            if book.config.is_multilang && book.config.is_main_book {
                match book.toc[0] {
                    TocItem::Numbered(ref i) |
                    TocItem::Unnumbered(ref i) |
                    TocItem::Unlisted(ref i) => {
                        let mut chapter: Chapter = i.chapter.clone();
                        chapter.dest_path = Some(PathBuf::from("index.html".to_string()));

                        // almost the same as process_chapter(), but we have to
                        // manipulate path_to_root in data and rendered_path

                        let mut content = try!(chapter.read_content_using(&book.config.src));

                        // Parse for playpen links
                        if let Some(p) = book.config.get_src().join(&chapter.path).parent() {
                            content = helpers::playpen::render_playpen(&content, p);
                        }

                        let mut data = try!(make_data(&book, &chapter, &book_project.livereload_script));

                        data.remove("path_to_root");
                        data.insert("path_to_root".to_owned(), "".to_json());

                        // Rendere the handlebars template with the data
                        debug!("[*]: Render template");
                        let rendered_content = try!(handlebars.render("page", &data));

                        let p = chapter.dest_path.unwrap();
                        let rendered_path = &book_project.get_dest_base().join(&p);

                        debug!("[*]: Create file {:?}", rendered_path);

                        // Write to file
                        let mut file = try!(utils::fs::create_file(rendered_path));
                        info!("[*] Creating {:?} ✓", rendered_path);

                        try!(file.write_all(&rendered_content.into_bytes()));
                    },
                    TocItem::Spacer => {},
                }
            }

            // Render a file for every entry in the book
            try!(self.process_items(&book.toc, &book, &book_project.livereload_script, &handlebars));
        }

        Ok(())
    }
}

impl HtmlHandlebars {

    fn process_items(&self,
                     items: &Vec<TocItem>,
                     book: &Book,
                     livereload_script: &Option<String>,
                     handlebars: &Handlebars)
                     -> Result<(), Box<Error>> {

        for item in items.iter() {
            match *item {
                TocItem::Numbered(ref i) |
                TocItem::Unnumbered(ref i) |
                TocItem::Unlisted(ref i) => {
                    // FIXME chapters with path "" are interpreted as draft now,
                    // not rendered here, and displayed gray in the TOC. Either
                    // path should be instead an Option or all chapter output
                    // should be used from setting dest_path, which is already
                    // Option but currently only used for rendering a chapter as
                    // index.html.
                    if i.chapter.path.as_os_str().len() > 0 {
                        try!(self.process_chapter(&i.chapter, book, livereload_script, handlebars));
                    }

                    if let Some(ref subs) = i.sub_items {
                        try!(self.process_items(&subs, book, livereload_script, handlebars));
                    }

                },
                _ => {},
            }
        }

        Ok(())
    }

    fn process_chapter(&self,
                       chapter: &Chapter,
                       book: &Book,
                       livereload_script: &Option<String>,
                       handlebars: &Handlebars)
                       -> Result<(), Box<Error>> {

        let data = try!(make_data(book, chapter, livereload_script));

        // Rendere the handlebars template with the data
        debug!("[*]: Render template");
        let rendered_content = try!(handlebars.render("page", &data));

        let p = match chapter.dest_path.clone() {
            Some(x) => x,
            None => chapter.path.with_extension("html")
        };

        let rendered_path = &book.config.get_dest().join(&p);

        debug!("[*]: Create file {:?}", rendered_path);

        // Write to file
        let mut file = try!(utils::fs::create_file(rendered_path));
        info!("[*] Creating {:?} ✓", rendered_path);

        try!(file.write_all(&rendered_content.into_bytes()));

        Ok(())
    }
}

fn make_data(book: &Book,
             chapter: &Chapter,
             livereload_script: &Option<String>)
             -> Result<serde_json::Map<String, serde_json::Value>, Box<Error>> {

    debug!("[fn]: make_data");

    let mut data = serde_json::Map::new();

    // Book data

    data.insert("language".to_owned(), "en".to_json());
    data.insert("title".to_owned(), book.config.title.to_json());
    data.insert("description".to_owned(), book.config.description.to_json());

    if let Some(ref x) = *livereload_script {
        data.insert("livereload".to_owned(), x.to_json());
    }

    // Chapter data

    let mut path = if let Some(ref dest_path) = chapter.dest_path {
        PathBuf::from(dest_path)
    } else {
        chapter.path.clone()
    };

    if book.config.is_multilang && path.as_os_str().len() > 0 {
        let p = PathBuf::from(&book.config.language.code);
        path = p.join(path);
    }

    match path.to_str() {
        Some(p) => {
            data.insert("path".to_owned(), p.to_json());
        },
        None => {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Could not convert path to str")
            ))
        },
    }

    let mut content = try!(chapter.read_content_using(&book.config.src));
    content = utils::render_markdown(&content);

    // Parse for playpen links
    if let Some(p) = book.config.get_src().join(&chapter.path).parent() {
        content = helpers::playpen::render_playpen(&content, p);
    }

    data.insert("content".to_owned(), content.to_json());

    data.insert("path_to_root".to_owned(), utils::fs::path_to_root(&path).to_json());

    if let Some(ref links) = chapter.translation_links {
        data.insert("translation_links".to_owned(), links.to_json());
    }

    let chapters = try!(items_to_chapters(&book.toc, &book));

    data.insert("chapters".to_owned(), chapters.to_json());
    debug!("[*]: JSON constructed");
    Ok(data)
}

fn items_to_chapters(items: &Vec<TocItem>, book: &Book)
                 -> Result<Vec<serde_json::Map<String, serde_json::Value>>, Box<Error>> {

    let mut chapters = vec![];

    for item in items.iter() {

        match *item {
            TocItem::Numbered(ref i) |
            TocItem::Unnumbered(ref i) => {
                match process_chapter_and_subs(i, book) {
                    Ok(mut x) => chapters.append(&mut x),
                    Err(e) => return Err(e),
                }
            },
            TocItem::Spacer => {
                let mut chapter = serde_json::Map::new();
                chapter.insert("spacer".to_owned(), "_spacer_".to_json());
                chapters.push(chapter);
            },
            TocItem::Unlisted(_) => {},
        }
    }

    Ok(chapters)
}

fn process_chapter_and_subs(i: &TocContent, book: &Book)
                            -> Result<Vec<serde_json::Map<String, serde_json::Value>>, Box<Error>> {

    let mut chapters = vec![];

    // Create the data to inject in the template
    let mut chapter = serde_json::Map::new();
    let ch = &i.chapter;

    if let Some(_) = i.section {
        let s = i.section_as_string();
        chapter.insert("section".to_owned(), s.to_json());
    }

    chapter.insert("title".to_owned(), ch.title.to_json());

    let mut path = if let Some(ref dest_path) = ch.dest_path {
        PathBuf::from(dest_path)
    } else {
        ch.path.clone()
    };

    if book.config.is_multilang && path.as_os_str().len() > 0 {
        let p = PathBuf::from(&book.config.language.code);
        path = p.join(path);
    }

    match path.to_str() {
        Some(p) => {
            chapter.insert("path".to_owned(), p.to_json());
        },
        None => {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Could not convert path to str")
            ))
        },
    }

    chapters.push(chapter);

    if let Some(ref subs) = i.sub_items {
        let mut sub_chs = try!(items_to_chapters(&subs, book));
        chapters.append(&mut sub_chs);
    }

    Ok(chapters)
}
