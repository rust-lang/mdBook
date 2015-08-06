use std::path::{Path, PathBuf, Component};
use std::error::Error;
use std::fs::{self, metadata, File};


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

    path.to_path_buf().parent().expect("")
        .components().fold(String::new(), |mut s, c| {
            match c {
                Component::Normal(_) => s.push_str("../"),
                _ => {
                    debug!("[*]: Other path component... {:?}", c);
                }
            }
            s
        })
}

/// This function checks for every component in a path if the directory exists,
/// if it does not it is created.

pub fn create_path(path: &Path) -> Result<(), Box<Error>> {
    debug!("[fn]: create_path");

    // Create directories if they do not exist
    let mut constructed_path = PathBuf::new();

    for component in path.components() {

        let mut dir;
        match component {
            Component::Normal(_) => { dir = PathBuf::from(component.as_os_str()); },
            Component::RootDir => {
                debug!("[*]: Root directory");
                // This doesn't look very compatible with Windows...
                constructed_path.push("/");
                continue
            },
            _ => continue,
        }

        constructed_path.push(&dir);
        debug!("[*]: {:?}", constructed_path);

        // Check if path exists
        match metadata(&constructed_path) {
            // Any way to combine the Err and first Ok branch ??
            Err(_) => {
                try!(fs::create_dir(&constructed_path));
                debug!("[*]: Directory created {:?}", constructed_path);
            },
            Ok(f) => {
                if !f.is_dir() {
                    try!(fs::create_dir(&constructed_path));
                    debug!("[*]: Directory created {:?}", constructed_path);
                } else {
                    debug!("[*]: Directory exists {:?}", constructed_path);
                    continue
                }
            },
        }
    }

    debug!("[*]: Constructed path: {:?}", constructed_path);

    Ok(())
}

/// This function creates a file and returns it. But before creating the file it checks every
/// directory in the path to see if it exists, and if it does not it will be created.

pub fn create_file(path: &Path) -> Result<File, Box<Error>> {
    debug!("[fn]: create_file");

    // Construct path
    if let Some(p) = path.parent() {
        try!(create_path(p));
    }

    debug!("[*]: Create file: {}", path);
    let f = try!(File::create(path));

    Ok(f)
}
