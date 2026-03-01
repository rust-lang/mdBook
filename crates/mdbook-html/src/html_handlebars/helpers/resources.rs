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
