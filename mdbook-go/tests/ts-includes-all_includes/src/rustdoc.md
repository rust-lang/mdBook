# Rustdoc Includes

## Rustdoc include adds the rest of the file as hidden

```rust
{{#rustdoc_include partially-included-test.rs:5:7}}
```

## Rustdoc include works with anchors too

```rust
{{#rustdoc_include partially-included-test-with-anchors.rs:rustdoc-include-anchor}}
```
