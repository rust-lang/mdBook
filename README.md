# mdBook

[![Build Status](https://github.com/rust-lang/mdBook/workflows/CI/badge.svg?event=push)](https://github.com/rust-lang/mdBook/actions?workflow=CI)
[![crates.io](https://img.shields.io/crates/v/mdbook.svg)](https://crates.io/crates/mdbook)
[![LICENSE](https://img.shields.io/github/license/rust-lang/mdBook.svg)](LICENSE)

mdBook is a utility to create modern online books from Markdown files.

This is a modified version that lets you support a custom playground backend other than rust.
Its entirely a "bring your own backend" situation so in the end you are responsible to safeguard against what kind of code
is running on your backend.

May or may not have some features broken due to mdBook being *VERY* hardcoded with rust in mind but should work just fine.

## Custom playground backend

For a custom playground backend you simply need a webserver with a `POST` route open. Call it whatever you like.

Then in the `.toml` config file of your book set the endpoint:
```toml
[output.html.playground]
editable = false                               # allows editing the source code
copyable = true                                # include the copy button for copying code snippets
copy-js = true                                 # includes the JavaScript for the code editor
line-numbers = true                            # displays line numbers for editable code
runnable = true                                # displays a run button for rust code
endpoint = "http://localhost:4242/playground/" # send the code to this url for execution
hidden-str = "#"                               # since different languange use certain chars
```

A clients incoming request looks as follows:
```json
{
	"lang": "cpp",
	"code": "..."
}
```

[See supported languanges](/guide/src/format/theme/syntax-highlighting.md) for syntax highlighting. As well as the `lang` options for incoming client requests.


A servers outgoing response should look as follows:
```json
{
	"result": "Request received!\n",
	"error": null
}
```

The client will display the appropriate message depending on the server's response.

## License

All the code in this repository is released under the ***Mozilla Public License v2.0***, for more information take a look at the [LICENSE] file.

[User Guide]: https://rust-lang.github.io/mdBook/
[contribution guide]: https://github.com/rust-lang/mdBook/blob/master/CONTRIBUTING.md
[LICENSE]: https://github.com/rust-lang/mdBook/blob/master/LICENSE
