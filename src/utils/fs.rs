use book::MDBook;

use std::path::{Path, PathBuf, Component};
use std::error::Error;
use std::io::{self, Read, Write};
use std::fs::{self, File};

use glob::{glob, Pattern};

use FILES;

/// Takes a path to a file and try to read the file into a String
pub fn file_to_string(path: &Path) -> Result<String, Box<Error>> {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            debug!("[*]: Failed to open {:?}", path);
            return Err(Box::new(e));
        },
    };

    let mut content = String::new();

    if let Err(e) = file.read_to_string(&mut content) {
        debug!("[*]: Failed to read {:?}", path);
        return Err(Box::new(e));
    }

    Ok(content)
}

/// Returns the contents of a static asset file by its path as &str. The path
/// should include "data/".
pub fn get_data_file(path: &str) -> Result<String, Box<Error>> {
    let content = match FILES.get(&path) {
        Ok(x) => String::from_utf8(x.into_owned()).unwrap_or("".to_string()),
        Err(e) => return Err(Box::new(e)),
    };
    Ok(content)
}

/// Writes the content of a data file from the embedded static assets to the
/// given destination path. Necessary folders will be created.
pub fn copy_data_file(src_path: &str, dest_path: &Path) -> Result<(), Box<Error>> {
    let content = match FILES.get(&src_path) {
        Ok(x) => x.into_owned(),
        Err(e) => return Err(Box::new(e)),
    };

    let mut f: File = try!(create_file(dest_path));

    match f.write_all(&content) {
        Ok(x) => Ok(x),
        Err(e) => Err(Box::new(e))
    }
}

/// Writes selected data files from the embedded static assets to the given
/// destination path.
///
/// `include_base` will be removed from the source path. This way the path
/// relative to the `dest_path` can be controlled.
///
/// The following will copy all files under "data/html-template/", excluding
/// folders that start with "_", take the "data/html-template/" part off the
/// source path, and write the entries to "assets" folder.
///
/// I.e. "data/html-template/css/book.css" will be written to
/// "assets/css/book.css".
///
/// ```no_run
/// utils::fs::copy_data("data/html-template/**/*",
///                      "data/html-template/",
///                      vec!["data/html-template/_*"],
///                      &Path::new("assets"));
/// ```
pub fn copy_data(include_glob: &str,
                 include_base: &str,
                 exclude_globs: Vec<&str>,
                 dest_base: &Path)
                 -> Result<(), Box<Error>> {

    let results = FILES.file_names()
        // narrow to files that match any of the include patterns
        .filter(|x| glob_matches(x, &vec![include_glob]))
        // exclude those which match any of the exclude patterns
        .filter(|x| !glob_matches(x, &exclude_globs))
        // copy each to the destination such that `include_base` is removed from the source path
        .map(|x| {
            let mut s: &str = &x.replace(include_base, "");
            s = s.trim_left_matches("/");

            let p = Path::new(s);
            let dest_path = dest_base.join(p);

            copy_data_file(x, &dest_path)
        })
        // only error results should remain
        .filter(|x| !x.is_ok());

    // collect errors as a String
    let mut s = String::new();
    for i in results {
        s.push_str(&format!("{:?}\n", i));
    }

    if s.len() > 1 as usize {
        Err(Box::new(io::Error::new(io::ErrorKind::Other, s)))
    } else {
        Ok(())
    }
}

/// Is there a match in any of the glob patterns?
pub fn glob_matches(text: &str, globs: &Vec<&str>) -> bool {
    let patterns = globs.iter().map(|x| Pattern::new(x).unwrap());
    for pat in patterns {
        if pat.matches(text) {
            return true;
        }
    }
    false
}

/// Same logic as `copy_data()` but operating on actual files instead of
/// embedded static assets.
pub fn copy_files(include_glob: &str,
                  include_base: &str,
                  exclude_globs: Vec<&str>,
                  dest_base: &Path)
                  -> Result<(), Box<Error>> {

    let pathbufs: Vec<PathBuf> = try!(glob(include_glob))
        .filter(|x| x.is_ok())
        .map(|x| x.unwrap())
        .collect::<Vec<PathBuf>>();

    let files = pathbufs.iter().filter_map(|x| x.to_str());

    let results =
    // narrow to files that match any of the include patterns
        files.filter(|x| glob_matches(x, &vec![include_glob]))
        // exclude those which match any of the exclude patterns
        .filter(|x| !glob_matches(x, &exclude_globs))
        // copy each to the destination such that `include_base` is removed from the source path
        .map(|x| {
            let mut s: &str = &x.replace(include_base, "");
            s = s.trim_left_matches("/");

            let p = Path::new(s);
            let dest_path = dest_base.join(p);

            // make sure parent exists
            if let Some(p) = dest_path.parent() {
                try!(fs::create_dir_all(p));
            }

            if dest_path.is_dir() {
                // if it is an already created dir
                Ok(0)
            } else {
                // this will error on folders, so don't try!() on results
                fs::copy(&x, &dest_path)
            }
        })
        // only error results should remain
        .filter(|x| !x.is_ok());

    // collect errors as a String
    let mut s = String::new();
    for i in results {
        s.push_str(&format!("{:?}\n", i));
    }

    if s.len() > 1 as usize {
        Err(Box::new(io::Error::new(io::ErrorKind::Other, s)))
    } else {
        Ok(())
    }
}

/// Takes a path and returns a path containing just enough `../` to point to the root of the given path.
///
/// This is mostly interesting for a relative path to point back to the directory from where the
/// path starts.
///
/// ```ignore
/// let mut path = Path::new("some/relative/path");
///
/// println!("{}", path_to_root(&path));
/// ```
///
/// **Outputs**
///
/// ```text
/// "../../"
/// ```
///
/// **note:** it's not very fool-proof, if you find a situation where it doesn't return the correct
/// path. Consider [submitting a new issue](https://github.com/azerupi/mdBook/issues) or a
/// [pull-request](https://github.com/azerupi/mdBook/pulls) to improve it.
pub fn path_to_root(path: &Path) -> String {
    debug!("[fn]: path_to_root");
    // Remove filename and add "../" for every directory

    path.to_path_buf()
        .parent()
        .expect("")
        .components()
        .fold(String::new(), |mut s, c| {
            match c {
                Component::Normal(_) => s.push_str("../"),
                _ => {
                    debug!("[*]: Other path component... {:?}", c);
                },
            }
            s
        })
}

/// This function creates a file and returns it. But before creating the file it checks every
/// directory in the path to see if it exists, and if it does not it will be created.
pub fn create_file(path: &Path) -> Result<File, Box<Error>> {
    debug!("[fn]: create_file");

    // Construct path
    if let Some(p) = path.parent() {
        debug!("Parent directory is: {:?}", p);

        try!(fs::create_dir_all(p));
    }

    debug!("[*]: Create file: {:?}", path);
    let f = match File::create(path) {
        Ok(f) => f,
        Err(e) => {
            debug!("File::create:    {}", e);
            return Err(Box::new(io::Error::new(io::ErrorKind::Other, format!("{}", e))));
        },
    };

    Ok(f)
}

// TODO why not just delete the folder and re-create it?

/// Removes all the content of a directory but not the directory itself
pub fn remove_dir_content(dir: &Path) -> Result<(), Box<Error>> {
    if !dir.exists() {
        return Ok(());
    }

    for item in try!(fs::read_dir(dir)) {
        if let Ok(item) = item {
            let item = item.path();
            if item.is_dir() {
                try!(fs::remove_dir_all(item));
            } else {
                try!(fs::remove_file(item));
            }
        }
    }
    Ok(())
}

/// Copies all files of a directory to another one except the files with the extensions given in the
/// `ext_blacklist` array
pub fn copy_files_except_ext(from: &Path, to: &Path, recursive: bool, ext_blacklist: &[&str]) -> Result<(), Box<Error>> {
    debug!("[fn] copy_files_except_ext");
    // Check that from and to are different
    if from == to {
        return Ok(());
    }
    debug!("[*] Loop");
    for entry in try!(fs::read_dir(from)) {
        let entry = try!(entry);
        debug!("[*] {:?}", entry.path());
        let metadata = try!(entry.metadata());

        // If the entry is a dir and the recursive option is enabled, call itself
        if metadata.is_dir() && recursive {
            if entry.path() == to.to_path_buf() {
                continue;
            }
            debug!("[*] is dir");

            // check if output dir already exists
            if !to.join(entry.file_name()).exists() {
                try!(fs::create_dir(&to.join(entry.file_name())));
            }

            try!(copy_files_except_ext(&from.join(entry.file_name()),
                                       &to.join(entry.file_name()),
                                       true,
                                       ext_blacklist));
        } else if metadata.is_file() {

            // Check if it is in the blacklist
            if let Some(ext) = entry.path().extension() {
                if ext_blacklist.contains(&ext.to_str().unwrap()) {
                    continue;
                }
            }
            debug!("[*] creating path for file: {:?}",
                   &to.join(entry.path().file_name().expect("a file should have a file name...")));

            info!("[*] Copying file: {:?}\n    to {:?}",
                  entry.path(),
                  &to.join(entry.path().file_name().expect("a file should have a file name...")));
            try!(fs::copy(entry.path(),
                          &to.join(entry.path().file_name().expect("a file should have a file name..."))));
        }
    }
    Ok(())
}

pub fn create_with_str(path: &PathBuf, text: &str) -> Result<File, String> {
    match File::create(path) {
        Err(e) => {
            return Err(format!("File doesn't exist, error in creating: {:?}", e));
        },
        Ok(mut f) => {
            let s = text.as_bytes();
            match f.write_all(s) {
                Ok(_) => Ok(f),
                Err(e) => Err(format!("File doesn't exist, error in writing: {:?}", e))
            }
        },
    }
}

/// Creates .gitignore in the project root folder.
pub fn create_gitignore(proj: &MDBook) {
    let gitignore = proj.get_project_root().join(".gitignore");

    if gitignore.exists() {
        return;
    }

    // Gitignore does not exist, create it

    // Figure out what is the user's output folder (can be default "book" or
    // custom config). This will be a full path, so remove the project_root from
    // it.
    let a = proj.get_project_root();
    let b = proj.get_dest_base();
    let c = b.strip_prefix(&a).unwrap();
    let relative_dest = c.to_str().expect("Path could not be yielded into a string slice.");

    debug!("[*]: {:?} does not exist, trying to create .gitignore", gitignore);

    let mut f = File::create(&gitignore).expect("Could not create file.");

    let text = format!("*.swp
.#*
*~
.DS_Store
{}", relative_dest);

    debug!("[*]: Writing to .gitignore");

    f.write_all(&text.into_bytes()).expect("Could not write to file.");
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------

// tests

#[cfg(test)]
mod tests {
    extern crate tempdir;

    use super::copy_files_except_ext;
    use std::fs;

    #[test]
    fn copy_files_except_ext_test() {
        let tmp = match tempdir::TempDir::new("") {
            Ok(t) => t,
            Err(_) => panic!("Could not create a temp dir"),
        };

        // Create a couple of files
        if let Err(_) = fs::File::create(&tmp.path().join("file.txt")) {
            panic!("Could not create file.txt")
        }
        if let Err(_) = fs::File::create(&tmp.path().join("file.md")) {
            panic!("Could not create file.md")
        }
        if let Err(_) = fs::File::create(&tmp.path().join("file.png")) {
            panic!("Could not create file.png")
        }
        if let Err(_) = fs::create_dir(&tmp.path().join("sub_dir")) {
            panic!("Could not create sub_dir")
        }
        if let Err(_) = fs::File::create(&tmp.path().join("sub_dir/file.png")) {
            panic!("Could not create sub_dir/file.png")
        }
        if let Err(_) = fs::create_dir(&tmp.path().join("sub_dir_exists")) {
            panic!("Could not create sub_dir_exists")
        }
        if let Err(_) = fs::File::create(&tmp.path().join("sub_dir_exists/file.txt")) {
            panic!("Could not create sub_dir_exists/file.txt")
        }

        // Create output dir
        if let Err(_) = fs::create_dir(&tmp.path().join("output")) {
            panic!("Could not create output")
        }
        if let Err(_) = fs::create_dir(&tmp.path().join("output/sub_dir_exists")) {
            panic!("Could not create output/sub_dir_exists")
        }

        match copy_files_except_ext(&tmp.path(), &tmp.path().join("output"), true, &["md"]) {
            Err(e) => panic!("Error while executing the function:\n{:?}", e),
            Ok(_) => {},
        }

        // Check if the correct files where created
        if !(&tmp.path().join("output/file.txt")).exists() {
            panic!("output/file.txt should exist")
        }
        if (&tmp.path().join("output/file.md")).exists() {
            panic!("output/file.md should not exist")
        }
        if !(&tmp.path().join("output/file.png")).exists() {
            panic!("output/file.png should exist")
        }
        if !(&tmp.path().join("output/sub_dir/file.png")).exists() {
            panic!("output/sub_dir/file.png should exist")
        }
        if !(&tmp.path().join("output/sub_dir_exists/file.txt")).exists() {
            panic!("output/sub_dir/file.png should exist")
        }

    }
}
