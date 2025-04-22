//! Tests for Rust playground support.

use crate::prelude::*;

// Verifies that a rust codeblock gets the playground class.
#[test]
fn playground_on_rust_code() {
    BookTest::from_dir("playground/playground_on_rust_code").check_main_file(
        "book/index.html",
        str![[r##"
<h1 id="rust-sample"><a class="header" href="#rust-sample">Rust Sample</a></h1>
<pre><pre class="playground"><code class="language-rust"><span class="boring">#![allow(unused)]
</span><span class="boring">fn main() {
</span>let x = 1;
<span class="boring">}</span></code></pre></pre>
"##]],
    );
}

// When the playground is disabled, there should be no playground class.
#[test]
fn disabled_playground() {
    BookTest::from_dir("playground/disabled_playground").check_main_file(
        "book/index.html",
        str![[r##"
<h1 id="rust-sample"><a class="header" href="#rust-sample">Rust Sample</a></h1>
<pre><code class="language-rust">let x = 1;</code></pre>
"##]],
    );
}
