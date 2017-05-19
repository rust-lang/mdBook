//! Configuration options that can be passed through environment variables.
//!
//! By default, mdBook will try to retrieve external resources through a content
//! delivery network. If that fails, it'll load an embedded version of the
//! resource. Using environment variables this behavior can be changed in
//! various ways.
//!
//! The variables are as follows. [name] can be one of JQUERY, MATHJAX,
//! HIGHLIGHT, AWESOME, OPEN_SANS or SOURCE_CODE_PRO:
//!
//! - MDBOOK_DEFAULT_STRATEGY: Strategy for finding external resources. One of
//!   UrlWithFallback (the default), EmbeddedOnly, UrlOnly and Omit.
//! - MDBOOK_[name]_URL: The URL of the resource. When omitted, an URL embedded at compile time will be used.
//! - MDBOOK_[name]_SOURCE: The source file for the embedded resource. When omitted, a version embeded at compile time will be used.
//! - MDBOOK_[name]_STRATEGY: Strategy for this particular resource. This
//!   overrides the global strategy.

use std::env;

pub static JQUERY_URL: &str = &"https://code.jquery.com/jquery-2.1.4.min.js";
pub static MATHJAX_URL: &str = &"https://cdnjs.cloudflare.com/ajax/libs/mathjax/2.7.1/MathJax.js?config=TeX-AMS-MML_HTMLorMML";
pub static HIGHLIGHT_URL: &str = &"highlight.js";
pub static AWESOME_URL: &str = &"https://maxcdn.bootstrapcdn.com/font-awesome/4.3.0/css/font-awesome.min.css";
pub static OPEN_SANS_URL: &str = &"https://fonts.googleapis.com/css?family=Open+Sans:300italic,400italic,600italic,700italic,800italic,400,300,600,700,800";
pub static SOURCE_CODE_PRO_URL: &str = &"https://fonts.googleapis.com/css?family=Source+Code+Pro:500";

pub enum Resource {
    UrlWithFallback {
        url: String,
        source: Option<String>
    },
    EmbeddedOnly {
        source: Option<String>
    },
    UrlOnly {
        url: String
    },
    Omit
}

impl Resource {
    pub fn must_render_url(&self) -> bool {
        match self {
            &Resource::UrlWithFallback{url: _, source: _} => true,
            &Resource::UrlOnly{url: _} => true,
            _ => false
        }
    }

    pub fn must_embed(&self) -> bool {
        match self {
            &Resource::UrlWithFallback{url: _, source: _} => true,
            &Resource::EmbeddedOnly{source: _} => true,
            _ => false
        }
    }

    pub fn url(&self) -> String {
        match self {
            &Resource::UrlWithFallback{url: ref url, source: _} => url.clone(),
            &Resource::UrlOnly{url: ref url} => url.clone(),
            _ => panic!("no url available")
        }
    }

    pub fn source(&self) -> Option<String> {
        match self {
            &Resource::UrlWithFallback{url: _, source: ref source} => source.clone(),
            &Resource::EmbeddedOnly{source: ref source} => source.clone(),
            _ => panic!("no source available")
        }
    }
}

pub struct Configuration {
    pub jquery: Resource,
    pub mathjax: Resource,
    pub highlight: Resource,
    pub awesome: Resource,
    pub open_sans: Resource,
    pub source_code_pro: Resource
}

fn varname(resource : &str, suffix: &str) -> String {
    "MDBOOK_".to_string() + &resource.to_uppercase() + "_" + suffix
}

fn var(resource: &str, key: &str) -> Option<String> {
    env::var(varname(resource, key)).ok()
}

fn var_default(resource: &str, key: &str, default: &str) -> String {
    var(resource, key).unwrap_or(String::from(default))
}

pub enum Strategy {
    UrlWithFallback,
    EmbeddedOnly,
    UrlOnly,
    Omit
}

fn strategy(resource: &str) -> Strategy {
    match var(resource, "STRATEGY")
        .unwrap_or(var_default("DEFAULT", "STRATEGY", "UrlWithFallback")).as_ref() {
            "EmbeddedOnly" => Strategy::EmbeddedOnly,
            "UrlOnly" => Strategy::UrlOnly,
            "Omit" => Strategy::Omit,
            _ => Strategy::UrlWithFallback
    }
}

fn url(resource: &str, default: &str) -> String {
    var_default(resource, "URL", default)
}

fn source(resource : &str) -> Option<String> {
    var(resource, "EMBED_SOURCE")
}

fn resource(resource: &str, url_default: &str) -> Resource {
    match strategy(resource) {
        Strategy::UrlWithFallback => Resource::UrlWithFallback {
            url: url(resource, url_default),
            source: source(resource)
        },
        Strategy::EmbeddedOnly => Resource::EmbeddedOnly {
            source: source(resource)
        },
        Strategy::UrlOnly => Resource::UrlOnly {
            url: url(resource, url_default)
        },
        Strategy::Omit => Resource::Omit
    }
}

pub fn configuration_from_env() -> Configuration {
    Configuration {
        jquery: resource("JQUERY", JQUERY_URL),
        mathjax: resource("MATHJAX", MATHJAX_URL),
        highlight: resource("HIGHLIGHT", HIGHLIGHT_URL),
        awesome: resource("AWESOME", AWESOME_URL),
        open_sans: resource("OPEN_SANS", OPEN_SANS_URL),
        source_code_pro: resource("SOURCE_CODE_PRO", SOURCE_CODE_PRO_URL)
    }
}
