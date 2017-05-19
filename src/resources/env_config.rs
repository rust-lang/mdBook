//! Configuration options that can be passed through environment variables.
//!
//! By default, mdBook will try to retrieve external resources through a content
//! delivery network. If that fails, it'll load an embedded version of the
//! resource. Using environment variables this behavior can be changed in
//! various ways.
//!
//! The variables are as follows. [name] can be one of JQUERY, MATHJAX,
//! HIGHLIGHT, AWESOME, OPEN_SANS, SOURCE_CODE_PRO:
//!
//! - MDBOOK_GLOBAL_STRATEGY: Strategy for finding external resources. One of
//!   UrlWithFallback (the default), UrlOnly and Omit.
//! - MDBOOK_[name]_URL: The URL of the resource.
//! - MDBOOK_[name]_SOURCE: The source file for the embedded resource.
//! - MDBOOK_[name]_STRATEGY: Strategy for this particular resource. This
//!   overrides the global strategy.
//!
//! All variables have sane defaults which were determined at compile time.
//! the SOURCE variables, when left empty, ensure that a version is used that
//! was embedded into mdBook itself at compile time.

use std::env;

pub static JQUERY_URL: &'static str = &"https://code.jquery.com/jquery-2.1.4.min.js";
pub static MATHJAX_URL: &'static str = &"https://cdnjs.cloudflare.com/ajax/libs/mathjax/2.7.1/MathJax.js?config=TeX-AMS-MML_HTMLorMML";
pub static HIGHLIGHT_URL: &'static str = &"highlight.js";
pub static AWESOME_URL: &'static str = &"https://maxcdn.bootstrapcdn.com/font-awesome/4.3.0/css/font-awesome.min.css";
pub static OPEN_SANS_URL: &'static str = &"https://fonts.googleapis.com/css?family=Open+Sans:300italic,400italic,600italic,700italic,800italic,400,300,600,700,800";
pub static SOURCE_CODE_PRO_URL: &'static str = &"https://fonts.googleapis.com/css?family=Source+Code+Pro:500";

/// A third-party resource
pub trait Resource {
    /// returns true if this resource has to be rendered as an url
    fn must_render_url(&self) -> bool;
    /// returns true if this resource has to be embedded in the book
    fn must_embed(&self) -> bool;
    /// returns the url for this resource. This panics if must_render_url returns false.
    fn url(&self) -> String;
    /// returns the source location for this resource, or None if none was configured.
    /// This panics if must_embed returns false.
    fn source(&self) -> Option<String>;
}

/// A third-party resource with nothing special to it.
pub enum BasicResource {
    UrlWithFallback {
        url: String,
        source: Option<String>
    },
    UrlOnly {
        url: String
    },
    Omit
}

impl Resource for BasicResource {
    fn must_render_url(&self) -> bool {
        match self {
            &BasicResource::UrlWithFallback{url: _, source: _} => true,
            &BasicResource::UrlOnly{url: _} => true,
            _ => false
        }
    }

    fn must_embed(&self) -> bool {
        match self {
            &BasicResource::UrlWithFallback{url: _, source: _} => true,
            _ => false
        }
    }

    fn url(&self) -> String {
        match self {
            &BasicResource::UrlWithFallback{ref url, source: _} => url.clone(),
            &BasicResource::UrlOnly{ref url} => url.clone(),
            _ => panic!("no url available")
        }
    }

    fn source(&self) -> Option<String> {
        match self {
            &BasicResource::UrlWithFallback{url: _, ref source} => source.clone(),
            _ => panic!("no source available")
        }
    }
}

/// Special struct for font-awesome, as it consists of multiple embedded files
/// but is configured as a directory. Methods implemented for this struct
/// calculate the paths of all the files in this directory.
pub struct Awesome {
    resource: BasicResource
}

impl Resource for Awesome {
    fn must_render_url(&self) -> bool {
        self.resource.must_render_url()
    }

    fn must_embed(&self) -> bool {
        self.resource.must_embed()
    }

    fn url(&self) -> String {
        self.resource.url()
    }

    fn source(&self) -> Option<String> {
        self.resource.source()
    }
}

impl Awesome {
    pub fn css_source(&self) -> Option<String> {
        self.resource.source().map(|p| p + "/css/font-awesome.min.css")
    }

    pub fn eot_source(&self) -> Option<String> {
        self.resource.source().map(|p| p + "/fonts/fontawesome-webfont.eot")
    }

    pub fn svg_source(&self) -> Option<String> {
        self.resource.source().map(|p| p + "/fonts/fontawesome-webfont.svg")
    }

    pub fn ttf_source(&self) -> Option<String> {
        self.resource.source().map(|p| p + "/fonts/fontawesome-webfont.ttf")
    }

    pub fn woff_source(&self) -> Option<String> {
        self.resource.source().map(|p| p + "/fonts/fontawesome-webfont.woff")
    }

    pub fn woff2_source(&self) -> Option<String> {
        self.resource.source().map(|p| p + "/fonts/fontawesome-webfont.woff2")
    }

    pub fn otf_source(&self) -> Option<String> {
        self.resource.source().map(|p| p + "/fonts/FontAwesome.otf")
    }
}

/// Configuration information for all the third-party resources. This specifies
/// for every resource whether it should be included at all, and if so, just as
/// an URL or also as an embedded resource.
pub struct Configuration {
    pub jquery: BasicResource,
    pub mathjax: BasicResource,
    pub highlight: BasicResource,
    pub awesome: Awesome,
    pub open_sans: BasicResource,
    pub source_code_pro: BasicResource
}

/// Calculate an environment variable name of the format MDBOOK_[name]_[key]
fn varname(resource : &str, key: &str) -> String {
    "MDBOOK_".to_string() + &resource.to_uppercase() + "_" + key
}

/// Returns the contents of the environment variable of the given resource with
/// the given key, or None if the environment variable was not set.
fn var(resource: &str, key: &str) -> Option<String> {
    env::var(varname(resource, key)).ok()
}

/// Returns a variable as var would. If the variable could not be found, return
/// the given default.
fn var_default(resource: &str, key: &str, default: &str) -> String {
    var(resource, key).unwrap_or(String::from(default))
}

/// The rendering strategy for a resource
enum Strategy {
    /// Render both an inclusion through an URL, and a fallback if it exists.
    UrlWithFallback,
    /// Render only the inclusion through an URL
    UrlOnly,
    /// Omit the resource altogether
    Omit
}

/// Return a strategy from an environment variable, or the default
/// `UrlWithFallback` if it can't be found.
fn strategy(resource: &str) -> Strategy {
    match var(resource, "STRATEGY")
        .unwrap_or(var_default("DEFAULT", "STRATEGY", "UrlWithFallback")).as_ref() {
            "UrlOnly" => Strategy::UrlOnly,
            "Omit" => Strategy::Omit,
            _ => Strategy::UrlWithFallback
    }
}

/// return an URL from an environment variable, or the given default if the URL
/// was not set.
fn url(resource: &str, default: &str) -> String {
    var_default(resource, "URL", default)
}

/// return a source location from an environment variable, if it can't be found.
fn source(resource : &str) -> Option<String> {
    var(resource, "EMBED_SOURCE")
}

/// return a resource from various environment variables.
fn resource(resource: &str, url_default: &str) -> BasicResource {
    match strategy(resource) {
        Strategy::UrlWithFallback => BasicResource::UrlWithFallback {
            url: url(resource, url_default),
            source: source(resource)
        },
        Strategy::UrlOnly => BasicResource::UrlOnly {
            url: url(resource, url_default)
        },
        Strategy::Omit => BasicResource::Omit
    }
}

/// parse font_awesome configuration from environment variables.
fn awesome() -> Awesome {
    Awesome { resource: resource("AWESOME", AWESOME_URL) }
}

/// Parse the configuration from the environment variables.
pub fn configuration_from_env() -> Configuration {
    Configuration {
        jquery: resource("JQUERY", JQUERY_URL),
        mathjax: resource("MATHJAX", MATHJAX_URL),
        highlight: resource("HIGHLIGHT", HIGHLIGHT_URL),
        awesome: awesome(),
        open_sans: resource("OPEN_SANS", OPEN_SANS_URL),
        source_code_pro: resource("SOURCE_CODE_PRO", SOURCE_CODE_PRO_URL)
    }
}
