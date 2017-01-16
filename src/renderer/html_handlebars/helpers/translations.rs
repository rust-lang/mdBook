use std::path::Path;
use std::collections::{VecDeque, BTreeMap};

use serde_json;
use handlebars::{Handlebars, HelperDef, RenderError, RenderContext, Helper, Context};

#[derive(Clone, Copy)]
pub struct TranslationLinksHelper;

impl HelperDef for TranslationLinksHelper {
    fn call(&self, c: &Context, _h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {

        let translation_links = c.navigate(rc.get_path(), &VecDeque::new(), "translation-links");
        let decoded: Vec<BTreeMap<String, String>> = serde_json::from_str(&translation_links.to_string()).unwrap();

        if decoded.len() == 0 {
            return Ok(());
        }

        try!(rc.writer.write("<ul class=\"translation-links\">".as_bytes()));

        for item in decoded {
            let empty = "".to_string();
            let link = item.get("link").unwrap_or(&empty);
            let code = item.get("code").unwrap_or(&empty);

            // None value in the link becomes an empty string
            let text = if link.len() == 0 {
                format!("<li><span>{}</span></li>", code)
            } else {
                format!("<li><a href=\"{}\">{}</a></li>", link, code)
            };

            try!(rc.writer.write(text.as_bytes()));
        }

        try!(rc.writer.write("</ul>".as_bytes()));

        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct TranslationIndexesHelper;

impl HelperDef for TranslationIndexesHelper {
    fn call(&self, c: &Context, _h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {

        let translation_indexes = c.navigate(rc.get_path(), &VecDeque::new(), "translation-indexes");
        let decoded: Vec<BTreeMap<String, String>> = serde_json::from_str(&translation_indexes.to_string()).unwrap();

        if decoded.len() == 0 {
            return Ok(());
        }

        try!(rc.writer.write("<ul class=\"translation-indexes\">".as_bytes()));

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
