# Mixed

This contains all tags randomly mixed together, to make sure style changes in one does not affect others.

### A heading

**Quite a Strong statement , to make**

~~No, cross that~~

> Whose **quote** is this
>
> > And ~~this~~
> >
> > > - and
> > > - this
> > > - also

```
You encountered a wild codepen
```

```rust,editable
// The codepen is editable and runnable
fn main(){
    println!("Hello world!");
}
```

<kbd>Ctrl</kbd> + <kbd>S</kbd> saves a file.

---

Although markdown does not have a native figure caption syntax, one can use block quote as a general-purpose "environment" to emulate it, and max-level heading inside the quote as caption:

> ![](https://rust-lang.org/logos/rust-logo-256x256-blk.png)
> ###### Figure 1: Our belived logo in glorious 256x256 resolution

Since the heading is inside the block quote, it will not pollute the document heading structure.

This is a lot more concise to write (and easier to read in plaintext!!) than the html `<figure>` environment, where the need for separate `<img/>` and `<figcaption>` tags adds significant visual noise.

---

- ~~An unordered list~~
- **Hello**
- _World_
- What
  1. Should
  2. be
  3. `put`
  4. here?
  5. **<kbd>Ctrl</kbd> + <kbd>S</kbd> saves a file.**

| col1 | col2 | col 3 | col 4 | col 5 | col 6 |
| ---- | ---- | ----- | ----- | ----- | ----- |
| val1 | val2 | val3  | val5  | val4  | val6  |

| col1 | col2 | col 3 | An Questionable table header | col 5 | col 6                                    |
| ---- | ---- | ----- | ---------------------------- | ----- | ---------------------------------------- |
| val1 | val2 | val3  | val5                         | val4  | An equally Questionable long table value |

### Things to do

- [x] Add individual tags
- [ ] Add language examples
- [ ] Add rust specific examples

And another image

![2018 rust-conf art svg](https://raw.githubusercontent.com/rust-lang/rust-artwork/461afe27d8e02451cf9f46e507f2c2a71d2b276b/2018-RustConf/lucy-mountain-climber.svg)
