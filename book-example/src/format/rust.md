# Rust code specific features

## Hiding code lines

There is a feature in mdBook that let's you hide code lines by prepending them with a `#`.

```bash
#fn main() {
    let x = 5;
    let y = 6;

    println!("{}", x + y);
#}
```

Will render as

```rust
#fn main() {
    let x = 5;
    let y = 7;

    println!("{}", x + y);
#}
```


## Inserting runnable Rust files

With the following syntax, you can insert runnable Rust files into your book:

```hbs
\{{#playpen file.rs}}
```

The path to the Rust file has to be relative from the current source file.

When play is clicked, the code snippet will be send to the [Rust Playpen]() to be compiled and run. The result is send back and displayed directly underneath the code.

Here is what a rendered code snippet looks like:

{{#playpen example.rs}}
