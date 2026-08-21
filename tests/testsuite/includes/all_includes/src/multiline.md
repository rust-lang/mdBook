# Multiline Includes

## Unindented

Simple inclusion

```rust
{{#include hello_world.rs}}
```

## Blockquote

> Quoted inclusion
> ```rust
> {{#include hello_world.rs}}
> ```

> [!NOTE]
> Inclusion in an admonition
> ```rust
> {{#include hello_world.rs}}
> ```

> Multiple levels
> > [!NOTE]
> > ```rust
> > {{#include hello_world.rs}}
> > ```

## Unordered List

- Indented once:
  ```rust
  {{#include hello_world.rs}}
  ```
  - Indented twice:
    ```rust
    {{#include hello_world.rs}}
    ```

## Ordered List

1. First, write a Hello, World program:
   ```rust
   {{#include hello_world.rs}}
   ```
1. Then, learn rest of Rust.

end
