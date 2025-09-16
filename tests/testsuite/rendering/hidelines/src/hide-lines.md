# Hide Lines

```python
~hidden()
nothidden():
~    hidden()
    ~hidden()
    nothidden()
```

```python,hidelines=!!!
!!!hidden()
nothidden():
!!!    hidden()
    !!!hidden()
    nothidden()
```

```rust
#![allow(something)]
#
#hidden();
# hidden();
## not_hidden();
#[not_hidden]
not_hidden();
```
