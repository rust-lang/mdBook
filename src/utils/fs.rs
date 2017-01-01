use std::path::{Path, Component};
use std::error::Error;
use std::io::{self, Read};
use std::fs::{self, File};

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

pub fn create_file(path: &Path) -> io::Result<File> {
    debug!("[fn]: create_file");

    // Construct path
    if let Some(p) = path.parent() {
        debug!("Parent directory is: {:?}", p);

        try!(fs::create_dir_all(p));
    }

    debug!("[*]: Create file: {:?}", path);
    File::create(path)
}

/// Removes all the content of a directory but not the directory itself

pub fn remove_dir_content(dir: &Path) -> Result<(), Box<Error>> {
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

///
///
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
