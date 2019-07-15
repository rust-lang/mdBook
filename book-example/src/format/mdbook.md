# mdBook-specific markdown

## Hiding code lines

There is a feature in mdBook that lets you hide code lines by prepending them
with a `#`.

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

mdBook will interpret included files as markdown. Since the include command
is usually used for inserting code snippets and examples, you will often
wrap the command with ```` ``` ```` to display the file contents without
interpretting them.

````hbs
```
\{{#include file.rs}}
```
````

## Including portions of a file
Often you only need a specific part of the file e.g. relevant lines for an
example. We support four different modes of partial includes:

```hbs
\{{#include file.rs:2}}
\{{#include file.rs::10}}
\{{#include file.rs:2:}}
\{{#include file.rs:2:10}}
```

The first command only includes the second line from file `file.rs`. The second
command includes all lines up to line 10, i.e. the lines from 11 till the end of
the file are omitted. The third command includes all lines from line 2, i.e. the
first line is omitted. The last command includes the excerpt of `file.rs`
consisting of lines 2 to 10.

To avoid breaking your book when modifying included files, you can also
include a specific section using anchors instead of line numbers.
An anchor is a pair of matching lines. The line beginning an anchor must
match the regex "ANCHOR:\s*[\w_-]+" and similarly the ending line must match
the regex "ANCHOR_END:\s*[\w_-]+". This allows you to put anchors in
any kind of commented line.

Consider the following file to include:
```rs
/* ANCHOR: all */

// ANCHOR: component
struct Paddle {
    hello: f32,
}
// ANCHOR_END: component

////////// ANCHOR: system
impl System for MySystem { ... }
////////// ANCHOR_END: system

/* ANCHOR_END: all */
```

Then in the book, all you have to do is:
````hbs
Here is a component:
```rust,no_run,noplaypen
\{{#include file.rs:component}}
```

Here is a system:
```rust,no_run,noplaypen
\{{#include file.rs:system}}
```

This is the full file.
```rust,no_run,noplaypen
\{{#include file.rs:all}}
```
````

Lines containing anchor patterns inside the included anchor are ignored.

## Inserting runnable Rust files

With the following syntax, you can insert runnable Rust files into your book:

```hbs
\{{#playpen file.rs}}
```

The path to the Rust file has to be relative from the current source file.

When play is clicked, the code snippet will be sent to the [Rust Playpen] to be
compiled and run. The result is sent back and displayed directly underneath the
code.

Here is what a rendered code snippet looks like:

{{#playpen example.rs}}

[Rust Playpen]: https://play.rust-lang.org/
