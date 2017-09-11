use std::path::Path;
use std::collections::BTreeMap;

use serde_json;
use handlebars::{Handlebars, RenderError, RenderContext, Helper, Renderable, Context};


// Handlebars helper for navigation

pub fn previous(_h: &Helper, r: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
    debug!("[fn]: previous (handlebars helper)");

    debug!("[*]: Get data from context");
    let chapters = rc.evaluate_absolute("chapters")
        .and_then(|c| {
                      serde_json::value::from_value::<Vec<BTreeMap<String, String>>>(c.clone())
                          .map_err(|_| RenderError::new("Could not decode the JSON data"))
                  })?;

    let current = rc.evaluate_absolute("path")?
        .as_str().ok_or_else(|| RenderError::new("Type error for `path`, string expected"))?
        .replace("\"", "");

    let mut previous: Option<BTreeMap<String, String>> = None;

    debug!("[*]: Search for current Chapter");
    // Search for current chapter and return previous entry
    for item in chapters {

        match item.get("path") {
            Some(path) if !path.is_empty() => {
                if path == &current {

                    debug!("[*]: Found current chapter");
                    if let Some(previous) = previous {

                        debug!("[*]: Creating BTreeMap to inject in context");
                        // Create new BTreeMap to extend the context: 'title' and 'link'
                        let mut previous_chapter = BTreeMap::new();

                        // Chapter title
                        previous
                            .get("name").ok_or_else(|| RenderError::new("No title found for chapter in JSON data"))
                            .and_then(|n| {
                                          previous_chapter.insert("title".to_owned(), json!(n));
                                          Ok(())
                                      })?;


                        // Chapter link
                        previous
                            .get("path").ok_or_else(|| RenderError::new("No path found for chapter in JSON data"))
                            .and_then(|p| {
                                Path::new(p)
                                    .with_extension("html")
                                    .to_str().ok_or_else(|| RenderError::new("Link could not be converted to str"))
                                    .and_then(|p| {
                                                  previous_chapter
                                                      .insert("link".to_owned(), json!(p.replace("\\", "/")));
                                                  Ok(())
                                              })
                            })?;


                        debug!("[*]: Render template");
                        // Render template
                        _h.template()
                            .ok_or_else(|| RenderError::new("Error with the handlebars template"))
                            .and_then(|t| {
                                          let mut local_rc = rc.with_context(Context::wraps(&previous_chapter)?);
                                          t.render(r, &mut local_rc)
                                      })?;
                    }
                    break;
                } else {
                    previous = Some(item.clone());
                }
            },
            _ => continue,

        }
    }

    Ok(())
}




pub fn next(_h: &Helper, r: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
    debug!("[fn]: next (handlebars helper)");

    debug!("[*]: Get data from context");
    let chapters = rc.evaluate_absolute("chapters")
        .and_then(|c| {
                      serde_json::value::from_value::<Vec<BTreeMap<String, String>>>(c.clone())
                          .map_err(|_| RenderError::new("Could not decode the JSON data"))
                  })?;
    let current = rc.evaluate_absolute("path")?
        .as_str().ok_or_else(|| RenderError::new("Type error for `path`, string expected"))?
        .replace("\"", "");

    let mut previous: Option<BTreeMap<String, String>> = None;

    debug!("[*]: Search for current Chapter");
    // Search for current chapter and return previous entry
    for item in chapters {

        match item.get("path") {

            Some(path) if !path.is_empty() => {

                if let Some(previous) = previous {

                    let previous_path = previous
                        .get("path").ok_or_else(|| RenderError::new("No path found for chapter in JSON data"))?;

                    if previous_path == &current {

                        debug!("[*]: Found current chapter");
                        debug!("[*]: Creating BTreeMap to inject in context");
                        // Create new BTreeMap to extend the context: 'title' and 'link'
                        let mut next_chapter = BTreeMap::new();

                        item.get("name").ok_or_else(|| RenderError::new("No title found for chapter in JSON data"))
                            .and_then(|n| {
                                          next_chapter.insert("title".to_owned(), json!(n));
                                          Ok(())
                                      })?;

                        Path::new(path)
                            .with_extension("html")
                            .to_str().ok_or_else(|| RenderError::new("Link could not converted to str"))
                            .and_then(|l| {
                                          debug!("[*]: Inserting link: {:?}", l);
                                          // Hack for windows who tends to use `\` as separator instead of `/`
                                          next_chapter.insert("link".to_owned(), json!(l.replace("\\", "/")));
                                          Ok(())
                                      })?;

                        debug!("[*]: Render template");

                        // Render template
                        _h.template().ok_or_else(|| RenderError::new("Error with the handlebars template"))
                            .and_then(|t| {
                                          let mut local_rc = rc.with_context(Context::wraps(&next_chapter)?);
                                          t.render(r, &mut local_rc)
                                      })?;
                        break;
                    }
                }

                previous = Some(item.clone());
            },

            _ => continue,
        }
    }
    Ok(())
}
