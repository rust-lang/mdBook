//! Theme dependencies for in-browser search. Not included in mdbook when
//! the "search" cargo feature is disabled.

pub static JS: &[u8] = include_bytes!("searcher.js");
pub static MARK_JS: &[u8] = include_bytes!("mark.min.js");
pub static ELASTICLUNR_JS: &[u8] = include_bytes!("elasticlunr.min.js");

#[cfg(feature = "search-non-english")]
pub mod lang {
    pub static STEMMER_SUPPORT_JS: &[u8] = include_bytes!("lunr.stemmer.support.js");
    pub static ARABIC_JS: &[u8] = include_bytes!("languages/lunr.ar.js");
    pub static CHINESE_JS: &[u8] = include_bytes!("languages/lunr.zh.js");
    pub static DANISH_JS: &[u8] = include_bytes!("languages/lunr.da.js");
    pub static DUTCH_JS: &[u8] = include_bytes!("languages/lunr.nl.js");
    pub static FINNISH_JS: &[u8] = include_bytes!("languages/lunr.fi.js");
    pub static FRENCH_JS: &[u8] = include_bytes!("languages/lunr.fr.js");
    pub static GERMAN_JS: &[u8] = include_bytes!("languages/lunr.de.js");
    pub static HUNGARIAN_JS: &[u8] = include_bytes!("languages/lunr.hu.js");
    pub static ITALIAN_JS: &[u8] = include_bytes!("languages/lunr.it.js");
    pub static JAPANESE_JS: &[u8] = include_bytes!("languages/lunr.ja.js");
    pub static KOREAN_JS: &[u8] = include_bytes!("languages/lunr.ko.js");
    pub static NORWEGIAN_JS: &[u8] = include_bytes!("languages/lunr.no.js");
    pub static PORTUGUESE_JS: &[u8] = include_bytes!("languages/lunr.pt.js");
    pub static ROMANIAN_JS: &[u8] = include_bytes!("languages/lunr.ro.js");
    pub static RUSSIAN_JS: &[u8] = include_bytes!("languages/lunr.ru.js");
    pub static SPANISH_JS: &[u8] = include_bytes!("languages/lunr.es.js");
    pub static SWEDISH_JS: &[u8] = include_bytes!("languages/lunr.sv.js");
    pub static TURKISH_JS: &[u8] = include_bytes!("languages/lunr.tr.js");
}
