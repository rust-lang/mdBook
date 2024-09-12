# Nested Chapter

This file has some testable code.

```rust
assert!($TEST_STATUS);
```

## Some Section

```rust
{{#include nested-test.rs}}
```

## Anchors include the part of a file between special comments

```rust
{{#include nested-test-with-anchors.rs:myanchor}}
```

## Includes can be shifted

```rust
{{#shiftinclude +2:nested-test-with-anchors.rs:myanchor}}
```

```rust
{{#shiftinclude auto:nested-test-with-anchors.rs:indentedanchor}}
```

## Rustdoc include adds the rest of the file as hidden

```rust
{{#rustdoc_include partially-included-test.rs:5:7}}
```

## Rustdoc include works with anchors too

```rust
{{#rustdoc_include partially-included-test-with-anchors.rs:rustdoc-include-anchor}}
```
