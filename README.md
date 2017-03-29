# mdBook

<table>
    <tr>
        <td><strong>Linux / OS X</strong></td>
        <td>
            <a href="https://travis-ci.org/azerupi/mdBook"><img src="https://travis-ci.org/azerupi/mdBook.svg?branch=master"></a>
        </td>
    </tr>
    <tr>
        <td><strong>Windows</strong></td>
        <td>
            <a href="https://ci.appveyor.com/project/azerupi/mdbook/"><img src="https://ci.appveyor.com/api/projects/status/o38racsnbcospyc8/branch/master?svg=true"></a>
        </td>
    </tr>
    <tr>
        <td colspan="2">
            <a href="https://crates.io/crates/mdbook"><img src="https://img.shields.io/crates/v/mdbook.svg"></a>
            <a href="LICENSE"><img src="https://img.shields.io/crates/l/mdbook.svg"></a>
        </td>
    </tr>
</table>

mdBook is a utility to create modern online books from Markdown files.

**This project is still evolving.**  
See [#90](https://github.com/azerupi/mdBook/issues/90)


## What does it look like?

The [**Documentation**](http://azerupi.github.io/mdBook/) for mdBook has been written in Markdown and is using mdBook to generate the online book-like website you can read. The documentation uses the latest version on GitHub and showcases the available features.

## Installation

There are multiple ways to install mdBook.

1. **Binaries**  
   Binaries are available for download [here](https://github.com/azerupi/mdBook/releases). Make sure to put the path to the binary into your `PATH`.

2. **From Crates.io**  
   This requires [Rust and Cargo](https://www.rust-lang.org/) to be installed. Once you have installed Rust, type the following in the terminal:
   ```
   cargo install mdbook
   ```

   This will download and compile mdBook for you, the only thing left to do is to add the Cargo bin directory to your `PATH`.

3. **From Git**  
   The version published to crates.io will ever so slightly be behind the version hosted here on GitHub. If you need the latest version you can build the git version of mdBook yourself. Cargo makes this ***super easy***!

   ```
   cargo install --git https://github.com/azerupi/mdBook.git
   ```
   Again, make sure to add the Cargo bin directory to your `PATH`.

4. **For Contributions**  
   If you want to contribute to mdBook you will have to clone the repository on your local machine:

   ```
   git clone https://github.com/azerupi/mdBook.git
   ```
   `cd` into `mdBook/` and run

   ```
   cargo build
   ```

   The resulting binary can be found in `mdBook/target/debug/` under the name `mdBook` or `mdBook.exe`.



## Usage

mdBook will primarily be used as a command line tool, even though it exposes all its functionality as a Rust crate for integration in other projects.

Here are the main commands you will want to run. For a more exhaustive explanation, check out the [documentation](http://azerupi.github.io/mdBook/).

- `mdbook init`

    The init command will create a directory with the minimal boilerplate to start with.

    ```
    book-test/
    ├── book
    └── src
        ├── chapter_1.md
        └── SUMMARY.md
    ```

    `book` and `src` are both directories. `src` contains the markdown files that will be used to render the output to the `book` directory.

    Please, take a look at the [**Documentation**](http://azerupi.github.io/mdBook/cli/init.html) for more information and some neat tricks.

- `mdbook build`

    This is the command you will run to render your book, it reads the `SUMMARY.md` file to understand the structure of your book, takes the markdown files in the source directory as input and outputs static html pages that you can upload to a server.

- `mdbook watch`

    When you run this command, mdbook will watch your markdown files to rebuild the book on every change. This avoids having to come back to the terminal to type `mdbook build` over and over again.

- `mdbook serve`

    Does the same thing as `mdbook watch` but additionally serves the book at `http://localhost:3000` (port is changeable) and reloads the browser when a change occurs.

### As a library

Aside from the command line interface, this crate can also be used as a library. This means that you could integrate it in an existing project, like a web-app for example. Since the command line interface is just a wrapper around the library functionality, when you use this crate as a library you have full access to all the functionality of the command line interface with an easy to use API and more!

See the [Documentation](http://azerupi.github.io/mdBook/lib/lib.html) and the [API docs](http://azerupi.github.io/mdBook/mdbook/index.html) for more information.

## Contributions

Contributions are highly appreciated and encouraged! Don't hesitate to participate to discussions in the issues, propose new features and ask for help.

If you are not very confident with Rust, **I will be glad to mentor as best as I can if you decide to tackle an issue or new feature.**

People who are not familiar with the code can look at [issues that are tagged **easy**](https://github.com/azerupi/mdBook/labels/Easy). A lot of issues are also related to web development, so people that are not comfortable with Rust can also participate! :wink:

You can pick any issue you want to work on. Usually it's a good idea to ask if someone is already working on it and if not to claim the issue.


## License

All the code is released under the ***Mozilla Public License v2.0***, for more information take a look at the [LICENSE](LICENSE) file.
