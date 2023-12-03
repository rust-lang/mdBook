use super::command_prelude::*;
use clap::builder::NonEmptyStringValueParser;
use mdbook::errors::Result;
use std::env;
use std::io::Cursor;
use syntect::dumps::dump_to_file;
use syntect::highlighting::ThemeSet;
use syntect::html::css_for_theme_with_class_style;
use syntect::html::ClassStyle;
use syntect::parsing::{SyntaxSet, SyntaxSetBuilder};

// Create clap subcommand arguments
pub fn make_subcommand() -> Command {
    Command::new("gen-syntax-cache")
        .about("Generate syntaxes.bin and css/syntax")
        .arg(
            Arg::new("dest-dir")
                .short('d')
                .long("dest-dir")
                .num_args(1)
                .default_value(".")
                .value_parser(NonEmptyStringValueParser::new())
                .help(
                    "Output directory for the syntax cache{n}\
                    Relative paths are interpreted relative to the current working directory.{n}\
                    If omitted, mdBook uses `.`.

                    This command outputs files [dir]/syntaxes.bin and [dir]/css/syntax/*.css",
                ),
        )
        .arg(
            Arg::new("dir")
                .long("dir")
                .num_args(1)
                .default_value(".")
                .value_parser(NonEmptyStringValueParser::new())
                .help(
                    "Root directory for the syntax sources{n}\
                    (Defaults to the Current Directory when omitted)",
                ),
        )
        .arg(arg!(--"syntaxes-only" "Only generate syntaxes.bin, not css/syntax/*.css."))
        .arg(arg!(--"no-default-syntaxes"
            "Don't include Sublime Text's default open source syntaxes{n}\
            If included, only syntaxes from [dir] are used."
        ))
        .arg(arg!(--"themes-only" "Only generate themes, not syntaxes.bin."))
        .arg(arg!(--"no-default-themes"
            "Don't include mdbook's default light, dark, and ayu themes{n}\
            If included, only themes from [dir] are used.'"
        ))
}

// Generate Syntax Cache command implementation
pub fn execute(args: &ArgMatches) -> Result<()> {
    let src_dir = env::current_dir()
        .unwrap()
        .join(format!("{}/", args.get_one::<String>("dir").unwrap()));
    let dest_dir = env::current_dir()
        .unwrap()
        .join(format!("{}/", args.get_one::<String>("dest-dir").unwrap()));

    if !args.get_flag("themes-only") {
        let mut builder = if args.get_flag("no-default-syntaxes") {
            SyntaxSetBuilder::new()
        } else {
            syntect::dumps::from_binary::<SyntaxSet>(mdbook::theme::SYNTAXES_BIN).into_builder()
        };
        builder.add_from_folder(&src_dir, true)?;
        let set = builder.build();
        for syntax in set.syntaxes() {
            info!(
                "supports syntax: {} [{}]",
                syntax.name,
                syntax.file_extensions.join(" ")
            );
        }
        dump_to_file(&set, dest_dir.join("syntaxes.bin"))?;
    }

    if !args.get_flag("syntaxes-only") {
        let mut builder = ThemeSet::load_from_folder(&src_dir)?;
        if !args.get_flag("no-default-themes") {
            if !builder.themes.contains_key("light") {
                let light = ThemeSet::load_from_reader(&mut Cursor::new(
                    mdbook::theme::SYNTAX_THEME_LIGHT,
                ))?;
                builder.themes.insert(String::from("light"), light);
            }
            if !builder.themes.contains_key("dark") {
                let dark =
                    ThemeSet::load_from_reader(&mut Cursor::new(mdbook::theme::SYNTAX_THEME_DARK))?;
                builder.themes.insert(String::from("dark"), dark);
            }
            if !builder.themes.contains_key("ayu") {
                let ayu =
                    ThemeSet::load_from_reader(&mut Cursor::new(mdbook::theme::SYNTAX_THEME_AYU))?;
                builder.themes.insert(String::from("ayu"), ayu);
            }
        }
        for (name, theme) in builder.themes.iter() {
            info!("supports theme: {}", name);
            std::fs::write(
                dest_dir.join(format!("css/syntax/{}.css", name)),
                css_for_theme_with_class_style(
                    theme,
                    ClassStyle::SpacedPrefixed { prefix: "syn-" },
                ),
            )?;
        }
    }

    Ok(())
}
