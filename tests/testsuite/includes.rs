//! Tests for include preprocessor.

use crate::prelude::*;

// Basic test for #include.
#[test]
fn include() {
    BookTest::from_dir("includes/all_includes")
        .check_main_file(
            "book/includes.html",
            str![[r##"
<h1 id="basic-includes"><a class="header" href="#basic-includes">Basic Includes</a></h1>
<h2 id="sample"><a class="header" href="#sample">Sample</a></h2>
<p>This is a sample include.</p>
"##]],
        )
        .check_main_file(
            "book/relative/includes.html",
            str![[r##"
<h1 id="relative-includes"><a class="header" href="#relative-includes">Relative Includes</a></h1>
<h2 id="sample"><a class="header" href="#sample">Sample</a></h2>
<p>This is a sample include.</p>
"##]],
        );
}

// Checks for anchored includes.
#[test]
fn anchored_include() {
    BookTest::from_dir("includes/all_includes").check_main_file(
        "book/anchors.html",
        str![[r##"
<h1 id="include-anchors"><a class="header" href="#include-anchors">Include Anchors</a></h1>
<pre><pre class="playground"><code class="language-rust"><span class="boring">#![allow(unused)]
</span><span class="boring">fn main() {
</span>let x = 1;
<span class="boring">}</span></code></pre></pre>
"##]],
    );
}
