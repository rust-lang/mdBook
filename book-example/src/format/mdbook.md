# mdBook-specific markdown

## Hiding code lines

There is a feature in mdBook that lets you hide code lines by prepending them with a `#`.

```bash
# fn main() {
    let x = 5;
    let y = 6;

    println!("{}", x + y);
# }
```

Will render as

```rust
# fn main() {
    let x = 5;
    let y = 7;

    println!("{}", x + y);
# }
```

## Including files

With the following syntax, you can include files into your book:

```hbs
\{{#include file.rs}}
```

The path to the file has to be relative from the current source file.

Usually, this command is used for including code snippets and examples. In this case, oftens one would include a specific part of the file e.g. which only contains the relevant lines for the example. We support four different modes of partial includes:

```hbs
\{{#include file.rs:2}}
\{{#include file.rs::10}}
\{{#include file.rs:2:}}
\{{#include file.rs:2:10}}
```

The first command only includes the second line from file `file.rs`. The second command includes all lines up to line 10, i.e. the lines from 11 till the end of the file are omitted. The third command includes all lines from line 2, i.e. the first line is omitted. The last command includes the excerpt of `file.rs` consisting of lines 2 to 10.

## Inserting runnable Rust files

With the following syntax, you can insert runnable Rust files into your book:

```hbs
\{{#playpen file.rs}}
```

The path to the Rust file has to be relative from the current source file.

When play is clicked, the code snippet will be sent to the [Rust Playpen] to be compiled and run. The result is sent back and displayed directly underneath the code.

Here is what a rendered code snippet looks like:

{{#playpen example.rs}}

[Rust Playpen]: https://play.rust-lang.org/
