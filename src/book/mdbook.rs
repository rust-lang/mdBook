use std::path::PathBuf;
use std::fs::{self, File, metadata};
use std::io::{Write, Result};

use book::bookconfig::BookConfig;
use book::bookitem::BookItem;
use parse;

pub struct MDBook {
    title: String,
    author: String,
    config: BookConfig,
    pub content: Vec<BookItem>
}

impl MDBook {

    pub fn new(path: &PathBuf) -> Self {

        // Hacky way to check if the path exists... Until PathExt moves to stable
        match metadata(path) {
            Err(_) => panic!("Directory does not exist"),
            Ok(f) => {
                if !f.is_dir() {
                    panic!("Is not a directory");
                }
            }
        }

        MDBook {
            title: String::from(""),
            author: String::from(""),
            content: vec![],
            config: BookConfig::new()
                        .set_src(path.join("src"))
                        .set_dest(path.join("book")),
        }
    }

    pub fn init(&self) -> Result<()> {

        let dest = self.config.dest();
        let src = self.config.src();

        // Hacky way to check if the directory exists... Until PathExt moves to stable
        match metadata(&dest) {
            Err(_) => {
                // There is a very high chance that the error is due to the fact that
                // the directory / file does not exist
                fs::create_dir(&dest).unwrap();
            },
            Ok(_) => { /* If there is no error, the directory / file does exist */ }
        }

        // Hacky way to check if the directory exists... Until PathExt moves to stable
        match metadata(&src) {
            Err(_) => {
                // There is a very high chance that the error is due to the fact that
                // the directory / file does not exist
                fs::create_dir(&src).unwrap();
            },
            Ok(_) => { /* If there is no error, the directory / file does exist */ }
        }

        // Hacky way to check if the directory exists... Until PathExt moves to stable
        let summary = match metadata(&src.join("SUMMARY.md")) {
            Err(_) => {
                // There is a very high chance that the error is due to the fact that
                // the directory / file does not exist
                Ok(File::create(&src.join("SUMMARY.md")).unwrap())
            },
            Ok(_) => {
                /* If there is no error, the directory / file does exist */
                Err("SUMMARY.md does already exist")
            }
        };

        if let Ok(mut f) = summary {
            try!(writeln!(f, "# Summary"));
            try!(writeln!(f, ""));
            try!(writeln!(f, "[Chapter 1](./chapter_1.md)"));

            let mut chapter_1 = File::create(&src.join("chapter_1.md")).unwrap();
            try!(writeln!(chapter_1, "# Chapter 1"));
        }

        return Ok(());
    }

    pub fn build(&mut self) -> Result<()> {

        try!(self.parse_summary());

        Ok(())
    }


    // Builder functions
    pub fn set_dest(mut self, dest: PathBuf) -> Self {
        self.config = self.config.set_dest(dest);
        self
    }

    pub fn set_src(mut self, src: PathBuf) -> Self {
        self.config = self.config.set_src(src);
        self
    }

    pub fn set_title(mut self, title: String) -> Self {
        self.title = title;
        self
    }

    pub fn set_author(mut self, author: String) -> Self {
        self.author = author;
        self
    }


    // Construct book
    fn parse_summary(&mut self) -> Result<()> {

        // When append becomes stale, use self.content.append() ...
        let book_items = try!(parse::construct_bookitems(&self.config.src().join("SUMMARY.md")));

        for item in book_items {
            self.content.push(item)
        }

        // Debug
        for item in &self.content {
            println!("name: \"{}\" path: {:?}", item.name, item.path);
        }

        Ok(())
    }

}
