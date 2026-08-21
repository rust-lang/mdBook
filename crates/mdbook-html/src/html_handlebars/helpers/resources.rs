use std::collections::HashMap;

use mdbook_core::utils;

use handlebars::{
    Context, Handlebars, Helper, HelperDef, Output, RenderContext, RenderError, RenderErrorReason,
};

// Handlebars helper to find filenames with hashes in them
#[derive(Clone)]
pub(crate) struct ResourceHelper {
    pub hash_map: HashMap<String, String>,
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

        // Honor an explicit `path_to_root` from the render data when present
        // (the `site-url` feature sets it to the absolute site root). Fall back
        // to deriving it from the page path, which is the depth-relative prefix
        // used for ordinary builds.
        let path_to_root = match rc.evaluate(ctx, "@root/path_to_root") {
            Ok(value) => value
                .as_json()
                .as_str()
                .map(|s| s.replace('"', ""))
                .unwrap_or_default(),
            Err(_) => String::new(),
        };
        let path_to_root = if path_to_root.is_empty() {
            let base_path = rc
                .evaluate(ctx, "@root/path")?
                .as_json()
                .as_str()
                .ok_or_else(|| {
                    RenderErrorReason::Other("Type error for `path`, string expected".to_owned())
                })?
                .replace("\"", "");
            utils::fs::path_to_root(&base_path)
        } else {
            path_to_root
        };

        out.write(&path_to_root)?;
        out.write(self.hash_map.get(param).map(|p| &p[..]).unwrap_or(&param))?;
        Ok(())
    }
}
