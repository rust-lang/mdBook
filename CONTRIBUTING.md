# Contributing

Welcome stranger!

If you have come here to learn how to contribute to mdBook, we have some tips for you!

First of all, don't hesitate to ask questions!
Use the [issue tracker](https://github.com/rust-lang-nursery/mdBook/issues), no question is too simple.
If we don't respond in a couple of days, ping us @Michael-F-Bryan, @budziq, @steveklabnik, @frewsxcv it might just be that we forgot. :wink:

### Issues to work on

Any issue is up for the grabbing, but if you are starting out, you might be interested in the
[E-Easy issues](https://github.com/rust-lang-nursery/mdBook/issues?q=is%3Aopen+is%3Aissue+label%3AE-Easy).
Those are issues that are considered more straightforward for beginners to Rust or the codebase itself.
These issues can be a good launching pad for more involved issues. Easy tasks for a first time contribution
include documentation improvements, new tests, examples, updating dependencies, etc.

If you come from a web development background, you might be interested in issues related to web technologies tagged
[A-JavaScript](https://github.com/rust-lang-nursery/mdBook/issues?q=is%3Aopen+is%3Aissue+label%3AA-JavaScript),
[A-Style](https://github.com/rust-lang-nursery/mdBook/issues?q=is%3Aopen+is%3Aissue+label%3AA-Style),
[A-HTML](https://github.com/rust-lang-nursery/mdBook/issues?q=is%3Aopen+is%3Aissue+label%3AA-HTML) or
[A-Mobile](https://github.com/rust-lang-nursery/mdBook/issues?q=is%3Aopen+is%3Aissue+label%3AA-Mobile).

When you decide you want to work on a specific issue, ping us on that issue so that we can assign it to you.
Again, do not hesitate to ask questions. We will gladly mentor anyone that want to tackle an issue.

Issues on the issue tracker are categorized with the following labels:

- **A**-prefixed labels state which area of the project an issue relates to.
- **E**-prefixed labels show an estimate of the experience necessary to fix the issue.
- **M**-prefixed labels are meta-issues used for questions, discussions, or tracking issues
- **S**-prefixed labels show the status of the issue
- **T**-prefixed labels show the type of issue

### Building mdBook

mdBook builds on stable Rust, if you want to build mdBook from source, here are the steps to follow:

1. Navigate to the directory of your choice
0. Clone this repository with git.

   ```
   git clone https://github.com/rust-lang-nursery/mdBook.git
   ```
0. Navigate into the newly created `mdBook` directory
0. Run `cargo build`

The resulting binary can be found in `mdBook/target/debug/` under the name `mdBook` or `mdBook.exe`.


### Making changes to the style

mdBook doesn't use CSS directly but uses [Stylus](http://stylus-lang.com/), a CSS-preprocessor which compiles to CSS.

When you want to change the style, it is important to not change the CSS directly because any manual modification to
the CSS files will be overwritten when compiling the stylus files. Instead, you should make your changes directly in the
[stylus files](https://github.com/rust-lang-nursery/mdBook/tree/master/src/theme/stylus) and regenerate the CSS.

For this to work, you first need [Node and NPM](https://nodejs.org/en/) installed on your machine.
Then run the following command to install both [stylus](http://stylus-lang.com/) and [nib](https://tj.github.io/nib/), you might need `sudo` to install successfully.

```
npm install -g stylus nib
```

When that finished, you can simply regenerate the CSS files by building mdBook with the following command:

```
cargo build --features=regenerate-css
```

This should automatically call the appropriate stylus command to recompile the files to CSS and include them in the project.

### Making a pull-request

When you feel comfortable that your changes could be integrated into mdBook, you can create a pull-request on GitHub.
One of the core maintainers will then approve the changes or request some changes before it gets merged.

If you want to make your pull-request even better, you might want to run [Clippy](https://github.com/Manishearth/rust-clippy)
and [rustfmt](https://github.com/rust-lang-nursery/rustfmt) on the code first.
This is not a requirement though and will never block a pull-request from being merged.

That's it, happy contributions! :tada: :tada: :tada:
