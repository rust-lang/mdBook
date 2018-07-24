use std::collections::BTreeMap;
use std::path::Path;

use handlebars::{Context, Handlebars, Helper, RenderContext, RenderError, Renderable, HelperResult, Output};
use serde_json;

use utils;

type StringMap = BTreeMap<String, String>;

/// Target for `find_chapter`.
enum Target {
    Previous,
    Next,
}

impl Target {
    /// Returns target if found.
    fn find(
        &self,
        base_path: &String,
        current_path: &String,
        current_item: &StringMap,
        previous_item: &StringMap,
    ) -> Result<Option<StringMap>, RenderError> {
        match self {
            &Target::Next => {
                let previous_path = previous_item
                    .get("path")
                    .ok_or_else(|| RenderError::new("No path found for chapter in JSON data"))?;

                if previous_path == base_path {
                    return Ok(Some(current_item.clone()));
                }
            }

            &Target::Previous => {
                if current_path == base_path {
                    return Ok(Some(previous_item.clone()));
                }
            }
        }

        Ok(None)
    }
}

fn find_chapter(cx: &Context, rc: &mut RenderContext, target: Target) -> Result<Option<StringMap>, RenderError> {
    debug!("Get data from context");

    let chapters = rc.evaluate_absolute(cx, "chapters", true).and_then(|c| {
        serde_json::value::from_value::<Vec<StringMap>>(c.clone())
            .map_err(|_| RenderError::new("Could not decode the JSON data"))
    })?;

    let base_path = rc.evaluate_absolute(cx, "path", true)?
        .as_str()
        .ok_or_else(|| RenderError::new("Type error for `path`, string expected"))?
        .replace("\"", "");

    let mut previous: Option<StringMap> = None;

    debug!("Search for chapter");

    for item in chapters {
        match item.get("path") {
            Some(path) if !path.is_empty() => {
                if let Some(previous) = previous {
                    if let Some(item) = target.find(&base_path, &path, &item, &previous)? {
                        return Ok(Some(item));
                    }
                }

                previous = Some(item.clone());
            }
            _ => continue,
        }
    }

    Ok(None)
}

fn render<'h: 'rc, 'rc>(
    h: &'h Handlebars, 
    cx: &'rc Context, 
    rc: &'rc mut RenderContext<'h>, 
    out: &mut Output,
    chapter: StringMap,
) -> HelperResult {
    trace!("Creating BTreeMap to inject in context");

    let mut context = BTreeMap::new();
    let base_path = rc.evaluate_absolute(cx, "path", false)?
        .as_str()
        .ok_or_else(|| RenderError::new("Type error for `path`, string expected"))?
        .replace("\"", "");

    context.insert(
        "path_to_root".to_owned(),
        json!(utils::fs::path_to_root(&base_path)),
    );

    chapter
        .get("name")
        .ok_or_else(|| RenderError::new("No title found for chapter in JSON data"))
        .map(|name| context.insert("title".to_owned(), json!(name)))?;

    chapter
        .get("path")
        .ok_or_else(|| RenderError::new("No path found for chapter in JSON data"))
        .and_then(|p| {
            Path::new(p)
                .with_extension("html")
                .to_str()
                .ok_or_else(|| RenderError::new("Link could not be converted to str"))
                .map(|p| context.insert("link".to_owned(), json!(p.replace("\\", "/"))))
        })?;

    trace!("Render template");

    h.render_template()
        .ok_or_else(|| RenderError::new("Error with the handlebars template"))
        .and_then(|t| {
            let local_cx = Context::wraps(&context)?;
            t.render(r, &local_cx, rc, out)
        })?;

    Ok(())
}

pub fn previous<'h: 'rc, 'rc>(
    h: &'h Handlebars, 
    cx: &'rc Context, 
    rc: &'rc mut RenderContext<'h>, 
    out: &mut Output
) -> HelperResult {
    trace!("previous (handlebars helper)");

    if let Some(previous) = find_chapter(cx, rc, Target::Previous)? {
        render(h, cx, rc, out, &previous)?;
    }

    Ok(())
}

pub fn next<'h: 'rc, 'rc>(
    h: &'h Handlebars, 
    cx: &'rc Context, 
    rc: &'rc mut RenderContext<'h>, 
    out: &mut Output
) -> HelperResult {
    trace!("next (handlebars helper)");

    if let Some(next) = find_chapter(cx, rc, Target::Next)? {
        render(h, cx, rc, out, &next)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEMPLATE: &'static str =
        "{{#previous}}{{title}}: {{link}}{{/previous}}|{{#next}}{{title}}: {{link}}{{/next}}";

    #[test]
    fn test_next_previous() {
        let data = json!({
         "name": "two",
         "path": "two.path",
         "chapters": [
            {
               "name": "one",
               "path": "one.path"
            },
            {
               "name": "two",
               "path": "two.path",
            },
            {
               "name": "three",
               "path": "three.path"
            }
         ]
      });

        let mut h = Handlebars::new();
        h.register_helper("previous", Box::new(previous));
        h.register_helper("next", Box::new(next));

        assert_eq!(
            h.render_template(TEMPLATE, &data).unwrap(),
            "one: one.html|three: three.html"
        );
    }

    #[test]
    fn test_first() {
        let data = json!({
         "name": "one",
         "path": "one.path",
         "chapters": [
            {
               "name": "one",
               "path": "one.path"
            },
            {
               "name": "two",
               "path": "two.path",
            },
            {
               "name": "three",
               "path": "three.path"
            }
         ]
      });

        let mut h = Handlebars::new();
        h.register_helper("previous", Box::new(previous));
        h.register_helper("next", Box::new(next));

        assert_eq!(
            h.render_template(TEMPLATE, &data).unwrap(),
            "|two: two.html"
        );
    }
    #[test]
    fn test_last() {
        let data = json!({
         "name": "three",
         "path": "three.path",
         "chapters": [
            {
               "name": "one",
               "path": "one.path"
            },
            {
               "name": "two",
               "path": "two.path",
            },
            {
               "name": "three",
               "path": "three.path"
            }
         ]
      });

        let mut h = Handlebars::new();
        h.register_helper("previous", Box::new(previous));
        h.register_helper("next", Box::new(next));

        assert_eq!(
            h.render_template(TEMPLATE, &data).unwrap(),
            "two: two.html|"
        );
    }
}
