use std::path::Path;
use std::collections::{VecDeque, BTreeMap};

use serde_json;
use handlebars::{Handlebars, HelperDef, RenderError, RenderContext, Helper, Context};

#[derive(Clone, Copy)]
pub struct TranslationsHelper;

impl HelperDef for TranslationsHelper {
    fn call(&self, c: &Context, _h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {

        let translation_links = c.navigate(rc.get_path(), &VecDeque::new(), "translation_links");
        let decoded: Vec<BTreeMap<String, String>> = serde_json::from_str(&translation_links.to_string()).unwrap();

        if decoded.len() == 0 {
            return Ok(());
        }

        try!(rc.writer.write("<ul class=\"translations\">".as_bytes()));

        for item in decoded {
            let empty = "".to_string();
            let link = item.get("link").unwrap_or(&empty);
            let code = item.get("code").unwrap_or(&empty);
            let text = format!("<li><a href=\"{}\">{}</a></li>", link, code);
            try!(rc.writer.write(text.as_bytes()));
        }

        try!(rc.writer.write("</ul>".as_bytes()));

        Ok(())
    }
}
