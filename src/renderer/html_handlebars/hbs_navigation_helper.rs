extern crate handlebars;
extern crate rustc_serialize;

use std::path::{PathBuf, Path};
use std::collections::BTreeMap;

use self::rustc_serialize::json::{self, ToJson};
use self::handlebars::{Handlebars, RenderError, RenderContext, Helper, Context, Renderable};

// Handlebars helper for navigation

pub fn previous(c: &Context, _h: &Helper, r: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
    debug!("[fn]: previous (handlebars helper)");

    debug!("[*]: Get data from context");
    // get value from context data
    // rc.get_path() is current json parent path, you should always use it like this
    // param is the key of value you want to display
    let chapters = c.navigate(rc.get_path(), "chapters");

    let current = c.navigate(rc.get_path(), "path")
        .to_string()
        .replace("\"", "");

    let path_to_root = PathBuf::from(
        c.navigate(rc.get_path(), "path_to_root")
            .to_string()
            .replace("\"", "")
    );

    debug!("[*]: Decode chapters from JSON");
    // Decode json format
    let decoded: Vec<BTreeMap<String, String>> = json::decode(&chapters.to_string()).unwrap();
    let mut previous: Option<BTreeMap<String, String>> = None;

    debug!("[*]: Search for current Chapter");
    // Search for current chapter and return previous entry
    for item in decoded {

        match item.get("path") {
            Some(path) if path.len() > 0 => {
                if path == &current {

                    debug!("[*]: Found current chapter");
                    if let Some(previous) = previous{

                        debug!("[*]: Creating BTreeMap to inject in context");
                        // Create new BTreeMap to extend the context: 'title' and 'link'
                        let mut previous_chapter = BTreeMap::new();

                        debug!("[*]: Inserting title: {}", previous.get("name").unwrap());
                        previous_chapter.insert("title".to_string(), previous.get("name").unwrap().to_json());

                        debug!("[*]: Inserting link: {}",
                            path_to_root.join(
                                Path::new(previous.get("path").unwrap())
                                    .with_extension("html")
                            ).to_str().unwrap());
                            
                        previous_chapter.insert(
                            "link".to_string(),
                            path_to_root.join(
                                Path::new(previous.get("path").unwrap())
                                    .with_extension("html")
                            ).to_str().unwrap().to_json()
                        );

                        debug!("[*]: Inject in context");
                        // Inject in current context
                        c.extend(&previous_chapter);

                        debug!("[*]: Render template");
                        // Render template
                        _h.template().unwrap().render(c, r, rc).unwrap();
                    }

                    break;
                }
                else {
                    previous = Some(item.clone());
                }
            },
            _ => continue,
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

    // Decode json format
    let decoded: Vec<BTreeMap<String,String>> = json::decode(&chapters.to_string()).unwrap();

    let mut is_current = false;

    for item in decoded {

        if let Some(path) = item.get("path") {
            if path.len() > 0 {
                if is_current {
                    let mut next = PathBuf::from(path);
                    next.set_extension("html");
                    try!(rc.writer.write(path_to_root.as_bytes()));
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
