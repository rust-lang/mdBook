use std::path::{Path, PathBuf, Component};
use std::error::Error;
use std::fs::{self, metadata};

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
