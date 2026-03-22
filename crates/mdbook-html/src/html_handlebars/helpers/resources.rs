use std::collections::HashMap;
use std::path::Path;

use crate::utils::clean_url_path_to_root;
use handlebars::{
    Context, Handlebars, Helper, HelperDef, Output, RenderContext, RenderError, RenderErrorReason,
};
use mdbook_core::utils;

// Handlebars helper to find filenames with hashes in them
#[derive(Clone)]
pub(crate) struct ResourceHelper {
    pub hash_map: HashMap<String, String>,
    pub no_html_extension: bool,
}

impl HelperDef for ResourceHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _r: &'reg Handlebars<'_>,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> Result<(), RenderError> {
        let param = h.param(0).and_then(|v| v.value().as_str()).ok_or_else(|| {
            RenderErrorReason::Other(
                "Param 0 with String type is required for resource helper.".to_owned(),
            )
        })?;

        let base_path = rc
            .evaluate(ctx, "@root/path")?
            .as_json()
            .as_str()
            .ok_or_else(|| {
                RenderErrorReason::Other("Type error for `path`, string expected".to_owned())
            })?
            .replace("\"", "");

        let path_to_root = if self.no_html_extension {
            clean_url_path_to_root(Path::new(&base_path))
        } else {
            utils::fs::path_to_root(&base_path)
        };

        out.write(&path_to_root)?;
        out.write(self.hash_map.get(param).map(|p| &p[..]).unwrap_or(&param))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use handlebars::Handlebars;
    use serde_json::json;

    #[test]
    fn resource_helper_without_clean_urls() {
        let helper = ResourceHelper {
            hash_map: HashMap::new(),
            no_html_extension: false,
        };

        let mut hbs = Handlebars::new();
        hbs.register_helper("resource", Box::new(helper));

        let data = json!({"path": "foo/bar.md"});
        let result = hbs
            .render_template("{{resource \"css/style.css\"}}", &data)
            .unwrap();

        // For foo/bar.md without clean URLs: path_to_root = "../"
        assert_eq!(result, "../css/style.css");
    }

    #[test]
    fn resource_helper_with_clean_urls() {
        let helper = ResourceHelper {
            hash_map: HashMap::new(),
            no_html_extension: true,
        };

        let mut hbs = Handlebars::new();
        hbs.register_helper("resource", Box::new(helper));

        let data = json!({"path": "foo/bar.md"});
        let result = hbs
            .render_template("{{resource \"css/style.css\"}}", &data)
            .unwrap();

        // For foo/bar.md with clean URLs: path_to_root = "../../" (extra level for bar/index.html)
        assert_eq!(result, "../../css/style.css");
    }

    #[test]
    fn resource_helper_clean_urls_index_page() {
        let helper = ResourceHelper {
            hash_map: HashMap::new(),
            no_html_extension: true,
        };

        let mut hbs = Handlebars::new();
        hbs.register_helper("resource", Box::new(helper));

        let data = json!({"path": "foo/index.md"});
        let result = hbs
            .render_template("{{resource \"css/style.css\"}}", &data)
            .unwrap();

        // For foo/index.md with clean URLs: path_to_root = "../" (no extra level for index pages)
        assert_eq!(result, "../css/style.css");
    }

    #[test]
    fn resource_helper_with_hash_map() {
        let mut hash_map = HashMap::new();
        hash_map.insert(
            "css/style.css".to_string(),
            "css/style-abc123.css".to_string(),
        );

        let helper = ResourceHelper {
            hash_map,
            no_html_extension: false,
        };

        let mut hbs = Handlebars::new();
        hbs.register_helper("resource", Box::new(helper));

        let data = json!({"path": "foo/bar.md"});
        let result = hbs
            .render_template("{{resource \"css/style.css\"}}", &data)
            .unwrap();

        // Should use hashed filename
        assert_eq!(result, "../css/style-abc123.css");
    }
}
