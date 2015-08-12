# mdBook ![Travis-CI](https://travis-ci.org/azerupi/mdBook.svg) [![Crates.io version](https://img.shields.io/crates/v/mdbook.svg)](https://crates.io/crates/mdbook) [![License](https://img.shields.io/crates/l/mdbook.svg)](LICENSE)

Personal implementation of Gitbook in Rust

**This project is still in it's early days.** 
For more information about what is left on my to-do list, check the issue tracker


## Example

To have an idea of what a rendered book looks like,take a look at the [**Documentation**](http://azerupi.github.io/mdBook/). It is rendered by the latest version of mdBook.

## Installation

```
git clone --depth=1 https://github.com/azerupi/mdBook.git
cd mdBook
cargo build --release
```

The executable `mdbook` will be in the `./target/release` folder, this should be added to the path.

## Structure

There are two main parts of this project:

- **The library:** The crate is structured so that all the code that actually does something is part of the library. You could therefore easily hook mdbook into your existing project, extend it's functionality by wrapping it in some other code, etc.
- **The binary:** Is just a wrapper around the library functionality providing a nice and easy command line interface.

### Command line interface

#### init

If you run `mdbook init` in a directory, it will create a couple of folders and files you can start with.
This is the strucutre it creates at the moment:
```
book-test/
├── book
└── src
    ├── chapter_1.md
    └── SUMMARY.md
```
`book` and `src` are both directories. `src` contains the markdown files that will be used to render the ouput to the `book` directory.

Please, take a look at the [**Documentation**](http://azerupi.github.io/mdBook/cli/init.html) for more information.

#### build

Use `mdbook build` in the directory to render the book. You can find more information in the [**Documentation**](http://azerupi.github.io/mdBook/cli/build.html)

### As a library

Aside from the command line interface, this crate can also be used as a library. This means that you could integrate it in an existing project, like a webapp for example. Since the command line interface is just a wrapper around the library functionality, when you use this crate as a library you have full access to all the functionality of the command line interface with and easy to use API and more!

See the [**Documentation**](http://azerupi.github.io/mdBook/lib/lib.html) and the [**API docs**](http://azerupi.github.io/mdBook/mdbook/index.html) for more information.

## Contributions

Contributions are highly apreciated. Here are some ideas:

- **Create new renderers**, at the moment I have only created a renderer that uses [handlebars](https://github.com/sunng87/handlebars-rust), [pulldown-cmark](https://github.com/google/pulldown-cmark) and renders to html. But you could create a renderer that uses another template engine, markdown parser or even outputs to another format like pdf.
- **Add tests** I have not much experience in writing tests, all help to write meaningful tests is thus very welcome
- **write documentation** documentation can always be improved
- **Smaller tasks** I try to add a lot of the remaining tasks on the issue tracker with the label: [`Enhancement`](https://github.com/azerupi/mdBook/issues?q=is%3Aopen+is%3Aissue+label%3AEnhancement). Just pick one that looks interesting. The majority of the tasks are small enough to be tackled by people who are unfamiliar with the project.

If you have an idea for improvement, create a new issue. Or a pull request if you can :)

## License

All the code is released under the ***Mozilla Public License v2.0***, for more information take a look at the [LICENSE](LICENSE) file
