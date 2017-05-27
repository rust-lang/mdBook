//! Resource management
//!
//! mdBook uses various third-party javascript libraries and fonts. This module
//! manages their resolution, dependent on run-time configuration, allowing the
//! renderer to determine how to include the resource.
//!
//! By default, resources are retrieved with an URL, usually a CDN. If this is not
//! possible, an embedded fallback is used instead. This improves rendering
//! speed while still allowing the book to be readable without an internet
//! connection.
//!
//! In some cases, this default behavior is unwanted. One might want to load the
//! library from a different URL, or one might want to embed a different
//! fallback resource. For those cases, resource resolution can be overridden
//! through environment variables. See the env_config module for more details.

mod env_config;
use self::env_config::{Configuration,configuration_from_env};
pub use self::env_config::{Resource};
use std::fs::File;
use std::io::Read;

static HIGHLIGHT_JS: &'static [u8] = include_bytes!("highlight.js");
static JQUERY: &'static [u8] = include_bytes!("jquery-2.1.4.min.js");
static FONT_AWESOME_CSS: &'static [u8] = include_bytes!("_FontAwesome/css/font-awesome.min.css");
static FONT_AWESOME_EOT: &'static [u8] = include_bytes!("_FontAwesome/fonts/fontawesome-webfont.eot");
static FONT_AWESOME_SVG: &'static [u8] = include_bytes!("_FontAwesome/fonts/fontawesome-webfont.svg");
static FONT_AWESOME_TTF: &'static [u8] = include_bytes!("_FontAwesome/fonts/fontawesome-webfont.ttf");
static FONT_AWESOME_WOFF: &'static [u8] = include_bytes!("_FontAwesome/fonts/fontawesome-webfont.woff");
static FONT_AWESOME_WOFF2: &'static [u8] = include_bytes!("_FontAwesome/fonts/fontawesome-webfont.woff2");
static FONT_AWESOME_OTF: &'static [u8] = include_bytes!("_FontAwesome/fonts/FontAwesome.otf");

/// handle for loading resource contents, based on a configuration
pub struct Resources {
    /// the configuration read from the environment variables
    pub conf: Configuration
}

impl Resources {
    /// create a new Resources object, based on the environment variables
    pub fn new() -> Resources {
        Resources {
            conf: configuration_from_env()
        }
    }

    /// Returns the contents that should be embedded in a book.
    /// This returns a version of the resource that was embedded at compile
    /// time, passed as the default argument, unless an alternative was set
    /// through an environment variable.
    fn resource_content(&self, default : &[u8]) -> Vec<u8> {
        if let Some(s) = self.conf.highlight.source() {
            if let Ok(mut f) = File::open(s) {
                let mut vec = Vec::new();
                f.read_to_end(&mut vec).expect("read failed");
                vec
            }
            else {
                Vec::from(default)
            }
        }
        else {
            Vec::from(default)
        }
    }

    pub fn highlight_js(&self) -> Vec<u8> {
        assert!(self.conf.highlight.must_embed());
        self.resource_content(HIGHLIGHT_JS)
    }

    pub fn jquery(&self) -> Vec<u8> {
        assert!(self.conf.jquery.must_embed());
        self.resource_content(JQUERY)
    }

    pub fn awesome_css(&self) -> Vec<u8> {
        assert!(self.conf.awesome.must_embed());
        self.resource_content(FONT_AWESOME_CSS)
    }
    pub fn awesome_eot(&self) -> Vec<u8> {
        assert!(self.conf.awesome.must_embed());
        self.resource_content(FONT_AWESOME_EOT)
    }
    pub fn awesome_svg(&self) -> Vec<u8> {
        assert!(self.conf.awesome.must_embed());
        self.resource_content(FONT_AWESOME_SVG)
    }
    pub fn awesome_ttf(&self) -> Vec<u8> {
        assert!(self.conf.awesome.must_embed());
        self.resource_content(FONT_AWESOME_TTF)
    }
    pub fn awesome_woff(&self) -> Vec<u8> {
        assert!(self.conf.awesome.must_embed());
        self.resource_content(FONT_AWESOME_WOFF)
    }
    pub fn awesome_woff2(&self) -> Vec<u8> {
        assert!(self.conf.awesome.must_embed());
        self.resource_content(FONT_AWESOME_WOFF2)
    }
    pub fn awesome_otf(&self) -> Vec<u8> {
        assert!(self.conf.awesome.must_embed());
        self.resource_content(FONT_AWESOME_OTF)
    }
}
