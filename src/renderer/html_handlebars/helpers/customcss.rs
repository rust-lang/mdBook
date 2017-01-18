use std::path::Path;
use std::collections::{VecDeque, BTreeMap};

use serde_json;
use handlebars::{Handlebars, HelperDef, RenderError, RenderContext, Helper, Context};

#[derive(Clone, Copy)]
pub struct CustomCssHelper;

impl HelperDef for CustomCssHelper {
    fn call(&self, c: &Context, _h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {

        let data = c.navigate(rc.get_path(), &VecDeque::new(), "custom-css-path");
        if let Ok(custom_css_path) = serde_json::from_str::<String>(&data.to_string()) {
            if custom_css_path.len() == 0 {
                return Ok(());
            }

            let p = custom_css_path.replace("\"", "");
            try!(rc.writer.write(format!("<link rel=\"stylesheet\" href=\"{}\">", p).as_bytes()));
        }

        Ok(())
    }
}
