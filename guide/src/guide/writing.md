# Writing code samples and documentation tests

If your book is about software, a short code sample may communicate the point better than many words of explanation.  
This section describes how to format samples and, perhaps more importantly, how to verify they compile and run
to ensue they stay aligned with the software APIs they describe.

Code blocks in your book are passed through mdBook and processed by rustdoc.  For more details on structuring codeblocks and running doc tests,
refer to the [rustdoc book](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html)

### Code blocks for sample code

You include a code sample in your book as a markdown fenced code block specifying `rust`, like so:

`````markdown
```rust
let four = 2 + 2;
assert_eq!(four, 4);
```
`````

This displays as:

```rust
let four = 2 + 2;
assert_eq!(four, 4);
```

Rustdoc will wrap this sample in a `fn main() {}` so that it can be compiled and even run by `mdbook test`.

#### Disable tests on a code block

rustdoc does not test code blocks which contain the `ignore` attribute:

`````markdown
```rust,ignore
fn main() {}
This would not compile anyway.
```
`````

rustdoc also doesn't test code blocks which specify a language other than Rust:

`````markdown
```markdown
**Foo**: _bar_
```
`````

rustdoc *does* test code blocks which have no language specified:

`````markdown
```
let four = 2 + 2;
assert_eq!(four, 4);
```
`````

### Hiding source lines within a sample

A longer sample may contain sections of boilerplate code that are not relevant to the current section of your book.
You can hide source lines within the code block prefixing them with `#_`
(that is a line starting with `#` followed by a single space), like so:

`````markdown
```rust
# use std::fs::File;
# use std::io::{Write,Result};
# fn main() -> Result<()> {
let mut file = File::create("foo.txt")?;
file.write_all(b"Hello, world!")?;
# Ok(())
# }
```
`````

This displays as:

```rust
# use std::fs::File;
# use std::io::{Write,Result};
# fn main() -> Result<()> {
let mut file = File::create("foo.txt")?;
file.write_all(b"Hello, world!")?;
# Ok(())
# }
```

Note that the code block displays an "show hidden lines" button in the upper right of the code block (when hovered over).

Note, too, that the sample provided its own `fn main(){}`, so the `use` statements could be positioned outside it.
When rustdoc sees the sample already provides `fn main`, it does *not* do its own wrapping.


### Tests using external crates

The previous example shows that you can `use` a crate within your sample.  
But if the crate is an *external* crate, that is, one declared as a dependency in your
package `Cargo.toml`, rustc (the compiler invoked by rustdoc) needs
`-L` and `--extern` switches in order to compile it.
Cargo does this automatically for `cargo build` and `cargo rustdoc` and mdBook can as well.

To allow mdBook to determine the correct external crate information,
add `package-dir` to your ***book.toml**, as described in [configuration](/format/configuration/general.md#rust-options).
Note that mdBook runs a `cargo build` for the package to determine correct dependencies.

This example (borrowed from the `serde` crate documentation) compiles and runs in a properly configured book:

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let point = Point { x: 1, y: 2 };

    // Convert the Point to a JSON string.
    let serialized = serde_json::to_string(&point).unwrap();

    // Prints serialized = {"x":1,"y":2}
    println!("serialized = {}", serialized);

    // Convert the JSON string back to a Point.
    let deserialized: Point = serde_json::from_str(&serialized).unwrap();

    // Prints deserialized = Point { x: 1, y: 2 }
    println!("deserialized = {:?}", deserialized);
}
```
