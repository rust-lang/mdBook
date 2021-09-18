use clap::{App, ArgMatches, SubCommand};
use mdbook::errors::Result;
use std::env;
use std::io::Cursor;
use syntect::dumps::dump_to_file;
use syntect::highlighting::ThemeSet;
use syntect::html::css_for_theme_with_class_style;
use syntect::html::ClassStyle;
use syntect::parsing::{SyntaxSet, SyntaxSetBuilder};

// Create clap subcommand arguments
pub fn make_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("gen-syntax-cache")
        .about("Generate syntaxes.bin and css/syntax")
        .arg_from_usage(
            "-d, --dest-dir=[dir] 'Output directory for the syntax cache{n}\
             Relative paths are interpreted relative to the current working directory.{n}\
             If omitted, mdBook uses `.`.
             This command outputs files [dir]/syntaxes.bin and [dir]/css/syntax/*.css'",
        )
        .arg_from_usage("--syntaxes-only 'Only generate syntaxes.bin, not css/syntax/*.css.'")
        .arg_from_usage(
            "--no-default-syntaxes 'Don't include Sublime Text's default open source syntaxes{n}\
             If included, only syntaxes from [dir] are used.'",
        )
        .arg_from_usage("--themes-only 'Only generate themes, not syntaxes.bin.'")
        .arg_from_usage(
            "--no-default-themes 'Don't include mdbook's default light, dark, and ayu themes{n}\
             If included, only themes from [dir] are used.'",
        )
        .arg_from_usage(
            "[dir] 'Root directory for the syntax sources{n}\
             (Defaults to the Current Directory when omitted)'",
        )
}

// Generate Syntax Cache command implementation
pub fn execute(args: &ArgMatches) -> Result<()> {
    let src_dir = env::current_dir()
        .unwrap()
        .join(&format!("{}/", args.value_of("dir").unwrap_or(".")));
    let dest_dir = env::current_dir()
        .unwrap()
        .join(&format!("{}/", args.value_of("dest-dir").unwrap_or(".")));

    if !args.is_present("themes-only") {
        let mut builder = if args.is_present("no-default-syntaxes") {
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

    if !args.is_present("syntaxes-only") {
        let mut builder = ThemeSet::load_from_folder(&src_dir)?;
        if !args.is_present("no-default-themes") {
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
