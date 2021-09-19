Console.sublime-syntax was written for mdBook.

Handlebars.sublime-syntax is from https://github.com/daaain/Handlebars/blob/master/grammars/Handlebars.tmLanguage (it was converted to `sublime-syntax` using Sublime Text).

TOML.sublime-syntax is from https://github.com/jasonwilliams/sublime_toml_highlighting/blob/0f14b7caf3c775a5d18551a9563a9debdb10c9a9/TOML.sublime-syntax

# Note

This folder is not copied over to the book directory when using `mdbook init`, nor is it indexed at runtime. All of the files in this folder are scraped by build.rs.

To make build.rs run again without running `cargo clean`, touch the run `touch build.rs`.

