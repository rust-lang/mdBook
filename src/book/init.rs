use std::path::PathBuf;
use config::Config;
use super::MDBook;
use errors::*;


/// A helper for setting up a new book and its directory structure.
#[derive(Debug, Clone, PartialEq)]
pub struct BookBuilder {
    root: PathBuf,
    create_gitignore: bool,
    config: Config,
    copy_theme: bool,
}

impl BookBuilder {
    pub fn new<P: Into<PathBuf>>(root: P) -> BookBuilder {
        BookBuilder {
            root: root.into(),
            create_gitignore: false,
            config: Config::default(),
            copy_theme: false,
        }
    }

    /// Set the `Config` to be used.
    pub fn with_config(&mut self, cfg: Config) -> &mut BookBuilder {
        self.config = cfg;
        self
    }

    pub fn copy_theme(&mut self, copy: bool) -> &mut BookBuilder {
        self.copy_theme = copy;
        self
    }

    pub fn create_gitignore(&mut self, create: bool) -> &mut BookBuilder {
        self.create_gitignore = create;
        self
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn build(&self) -> Result<MDBook> {
        unimplemented!()
    }
}

// contents of old `init()` function:

// debug!("[fn]: init");

// if !self.root.exists() {
//     fs::create_dir_all(&self.root).unwrap();
//     info!("{:?} created", self.root.display());
// }

// {
//     let dest = self.get_destination();
//     if !dest.exists() {
// debug!("[*]: {} does not exist, trying to create directory",
// dest.display());         fs::create_dir_all(dest)?;
//     }


//     let src = self.get_source();
//     if !src.exists() {
// debug!("[*]: {} does not exist, trying to create directory",
// src.display());         fs::create_dir_all(&src)?;
//     }

//     let summary = src.join("SUMMARY.md");

//     if !summary.exists() {
//         // Summary does not exist, create it
// debug!("[*]: {:?} does not exist, trying to create SUMMARY.md",
// &summary);         let mut f = File::create(&summary)?;

//         debug!("[*]: Writing to SUMMARY.md");

//         writeln!(f, "# Summary")?;
//         writeln!(f, "")?;
//         writeln!(f, "- [Chapter 1](./chapter_1.md)")?;
//     }
// }

// // parse SUMMARY.md, and create the missing item related file
// self.parse_summary()?;

// debug!("[*]: constructing paths for missing files");
// for item in self.iter() {
//     debug!("[*]: item: {:?}", item);
//     let ch = match *item {
//         BookItem::Spacer => continue,
//         BookItem::Chapter(_, ref ch) | BookItem::Affix(ref ch) => ch,
//     };
//     if !ch.path.as_os_str().is_empty() {
//         let path = self.get_source().join(&ch.path);

//         if !path.exists() {
//             if !self.create_missing {
//                 return Err(
// format!("'{}' referenced from SUMMARY.md does not
// exist.", path.to_string_lossy()).into(),                 );
//             }
//             debug!("[*]: {:?} does not exist, trying to create file", path);
//             ::std::fs::create_dir_all(path.parent().unwrap())?;
//             let mut f = File::create(path)?;

//             // debug!("[*]: Writing to {:?}", path);
//             writeln!(f, "# {}", ch.name)?;
//         }
//     }
// }

// debug!("[*]: init done");
// Ok(())

// pub fn copy_theme(&self) -> Result<()> {
//     debug!("[fn]: copy_theme");

//     let themedir = self.theme_dir();

//     if !themedir.exists() {
// debug!("[*]: {:?} does not exist, trying to create directory",
// themedir);         fs::create_dir(&themedir)?;
//     }

//     // index.hbs
//     let mut index = File::create(themedir.join("index.hbs"))?;
//     index.write_all(theme::INDEX)?;

//     // book.css
//     let mut css = File::create(themedir.join("book.css"))?;
//     css.write_all(theme::CSS)?;

//     // favicon.png
//     let mut favicon = File::create(themedir.join("favicon.png"))?;
//     favicon.write_all(theme::FAVICON)?;

//     // book.js
//     let mut js = File::create(themedir.join("book.js"))?;
//     js.write_all(theme::JS)?;

//     // highlight.css
//     let mut highlight_css = File::create(themedir.join("highlight.css"))?;
//     highlight_css.write_all(theme::HIGHLIGHT_CSS)?;

//     // highlight.js
//     let mut highlight_js = File::create(themedir.join("highlight.js"))?;
//     highlight_js.write_all(theme::HIGHLIGHT_JS)?;

//     Ok(())
// }
