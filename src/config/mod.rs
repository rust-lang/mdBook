pub mod bookconfig;
pub mod htmlconfig;
pub mod tomlconfig;

// Re-export the config structs
pub use self::bookconfig::BookConfig;
pub use self::htmlconfig::HtmlConfig;
