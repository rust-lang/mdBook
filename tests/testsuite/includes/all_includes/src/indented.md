# Indented Includes

 - No include:
   ## Sample
   
   This is not an include.

 - Basic include:
   {{#include sample.md}}

 - No include:
   ```rust
   fn main() {
       some_function();
   }
   ```

 - Partial include:
   ```rust
   {{#include partially-included-test.rs:5:7}}
   ```

 - No include:
   ```rust
   # fn some_function() {
   #     println!("some function");
   # }
   # 
   fn main() {
       some_function();
   }
   ```

 - Rustdoc include:
   ```rust
   {{#rustdoc_include partially-included-test.rs:5:7}}
   ```
