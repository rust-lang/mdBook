# Multiline Include Issues

## 619

```rust
{{#include hidden_lines_file.txt::2}}
# {{#include hidden_lines_file.txt:3:10}}
{{#include hidden_lines_file.txt:11:}}
```

## 1127

### Blockquote with included code inside

Lorum ipsum dolor sit amet.

> But do included code blocks...?
>
> ```rust
> {{#include another-included-test.rs}}
> ```
>
> By Quote Author

## 1564

### sample.md

- Example without including files.

  ```rust
  struct MyStruct;

  impl MyStruct {
      pub fn myfn() -> () {
          println!("MyStruct#myfn()");
      }
  }

  pub fn main() -> () {
      println!("main()");
  }
  ```

- Example using including files.

  ```rust
  struct MyStruct;

  impl MyStruct {
      {{#include myfn.rs}}
  }

  {{#include main.rs}}
  ```

## 1626

### Exercises

1. In the the code below:
    ```haskell
    {{#include Solutions.purs:eqNonEmpty}}
    ```
    we ...

## 2521

### Chapter 1

- This is demo
  ```rust
  {{ #include linear_search.rs }}
  ```
- Hello World