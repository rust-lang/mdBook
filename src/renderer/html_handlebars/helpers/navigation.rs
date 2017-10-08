use std::path::Path;

use serde_json;
use handlebars::{Context, Handlebars, Helper, RenderContext, RenderError, Renderable};

fn find_chapter<'a>(chapter: &'a serde_json::Value, needle: &str) -> Result<Option<&'a serde_json::Map<String, serde_json::Value>>, RenderError> {
    let chapter = chapter.as_object()
        .ok_or_else(|| RenderError::new("Chapter is not an object"))?;

    if let Some(link) = chapter.get("link") {
        if link == needle {
            return Ok(Some(From::from(chapter)));
        }
    }

    if let Some(children) = chapter.get("children") {
        let children = children.as_array()
            .ok_or_else(|| RenderError::new("chapter.children is not an array"))?;

        for child in children {
            if let Some(result) = find_chapter(child, needle)? {
                return Ok(Some(result));
            }
        }
    }

    Ok(None)
}

fn current_chapter<'a>(rc: &'a RenderContext) -> Result<Option<&'a serde_json::Map<String, serde_json::Value>>, RenderError> {
    let toc = rc.evaluate_absolute("toc")
        .map_err(|_| RenderError::new("Could not find toc in context"))?;

    let current_path = rc.evaluate_absolute("path")?
        .as_str()
        .ok_or_else(|| RenderError::new("Type error for `path`, string expected"))?
        .replace("\"", "");

    let current_link = Path::new(&current_path)
        .with_extension("html")
        .to_str()
        .unwrap()
        .replace("\\", "/");

    println!("Looking for \"{}\"", current_link);
    find_chapter(toc, &current_link)
}

fn nav_link(h: &Helper, r: &Handlebars, rc: &mut RenderContext, nav_obj_key: &str) -> Result<(), RenderError> {
    if let Some(current) = current_chapter(rc)?.map(|v| v.clone()) {
        if let Some(nav_obj) = current.get(nav_obj_key) {
            h.template()
                .ok_or_else(|| RenderError::new("Error with the handlebars template"))
                .and_then(|t| {
                    let mut local_rc = rc.with_context(Context::wraps(&nav_obj)?);
                    t.render(r, &mut local_rc)
                })?;
        }
    }

    Ok(())
}

pub fn previous(h: &Helper, r: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
    nav_link(h, r, rc, "previous")
}

pub fn next(h: &Helper, r: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
    nav_link(h, r, rc, "next")
}
