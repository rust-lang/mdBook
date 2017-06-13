pub mod bookconfig;
pub mod htmlconfig;
pub mod tomlconfig;
pub mod jsonconfig;

// Re-export the config structs
pub use self::bookconfig::BookConfig;
pub use self::htmlconfig::HtmlConfig;
pub use self::tomlconfig::TomlConfig;
