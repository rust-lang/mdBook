pub mod bookconfig;
pub mod htmlconfig;
pub mod playpenconfig;
pub mod tomlconfig;
pub mod jsonconfig;

// Re-export the config structs
pub use self::bookconfig::BookConfig;
pub use self::htmlconfig::HtmlConfig;
pub use self::playpenconfig::PlaypenConfig;
pub use self::tomlconfig::TomlConfig;
