extern crate handlebars;
extern crate rustc_serialize;

use std::path::{PathBuf};
use std::collections::BTreeMap;

use self::rustc_serialize::json;
use self::handlebars::{Handlebars, RenderError, RenderContext, Helper, Context};

// Handlebars helper for navigation

pub fn previous(c: &Context, _h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {

    // get value from context data
    // rc.get_path() is current json parent path, you should always use it like this
    // param is the key of value you want to display
    let chapters = c.navigate(rc.get_path(), "chapters");
    let current = c.navigate(rc.get_path(), "path").to_string().replace("\"", "");
    let path_to_root = c.navigate(rc.get_path(), "path_to_root").to_string().replace("\"", "");

    try!(rc.writer.write(path_to_root.as_bytes()));

    // Decode json format
    let decoded: Vec<BTreeMap<String,String>> = json::decode(&chapters.to_string()).unwrap();

    let mut previous = PathBuf::new();

    for item in decoded {

        if let Some(path) = item.get("path") {
            previous = if path.len() > 0 {

                if path == &current {
                    previous.set_extension("html");
                    try!(rc.writer.write(previous.to_str().unwrap().as_bytes()));
                    break;
                }

                PathBuf::from(path)

            } else { previous }
        }
    }

    Ok(())
}



pub fn next(c: &Context, _h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {

    // get value from context data
    // rc.get_path() is current json parent path, you should always use it like this
    // param is the key of value you want to display
    let chapters = c.navigate(rc.get_path(), "chapters");
    let current = c.navigate(rc.get_path(), "path").to_string().replace("\"", "");
    let path_to_root = c.navigate(rc.get_path(), "path_to_root").to_string().replace("\"", "");

    try!(rc.writer.write(path_to_root.as_bytes()));

    // Decode json format
    let decoded: Vec<BTreeMap<String,String>> = json::decode(&chapters.to_string()).unwrap();

    let mut is_current = false;

    for item in decoded {

        if let Some(path) = item.get("path") {
            if path.len() > 0 {
                if is_current {
                    let mut next = PathBuf::from(path);
                    next.set_extension("html");
                    try!(rc.writer.write(next.to_str().unwrap().as_bytes()));
                    break;
                } else if path == &current {
                    is_current = true;
                }
            }
        }
    }

    Ok(())
}
