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

// Checks behavior of recursive include.
#[test]
fn recursive_include() {
    BookTest::from_dir("includes/all_includes")
        .run("build", |cmd| {
            cmd.expect_stderr(str![[r#"
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Book building has started
[TIMESTAMP] [ERROR] (mdbook_driver::builtin_preprocessors::links): Stack depth exceeded in recursive.md. Check for cyclic includes
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Running the html backend
[TIMESTAMP] [INFO] (mdbook_html::html_handlebars::hbs_renderer): HTML book written to `[ROOT]/book`

"#]]);
        })
        .check_main_file(
            "book/recursive.html",
            str![[r#"
<p>Around the world, around the world
Around the world, around the world
Around the world, around the world
Around the world, around the world
Around the world, around the world
Around the world, around the world
Around the world, around the world
Around the world, around the world
Around the world, around the world
Around the world, around the world
Around the world, around the world</p>
"#]],
        );
}

// Checks the behavior of `{{#playground}}` include.
#[test]
fn playground_include() {
    BookTest::from_dir("includes/all_includes")
        .check_main_file("book/playground.html",
            str![[r##"
<h1 id="playground-includes"><a class="header" href="#playground-includes">Playground Includes</a></h1>
<pre><pre class="playground"><code class="language-rust">fn main() {
    println!("Hello World!");
<span class="boring">
</span><span class="boring">   // You can even hide lines! :D
</span><span class="boring">  println!("I am hidden! Expand the code snippet to see me");
</span>}</code></pre></pre>
"##]]);
}

// Checks the behavior of `{{#rustdoc_include}}`.
#[test]
fn rustdoc_include() {
    BookTest::from_dir("includes/all_includes")
        .check_main_file("book/rustdoc.html",
            str![[r##"
<h1 id="rustdoc-includes"><a class="header" href="#rustdoc-includes">Rustdoc Includes</a></h1>
<h2 id="rustdoc-include-adds-the-rest-of-the-file-as-hidden"><a class="header" href="#rustdoc-include-adds-the-rest-of-the-file-as-hidden">Rustdoc include adds the rest of the file as hidden</a></h2>
<pre><pre class="playground"><code class="language-rust"><span class="boring">fn some_function() {
</span><span class="boring">    println!("some function");
</span><span class="boring">}
</span><span class="boring">
</span>fn main() {
    some_function();
}</code></pre></pre>
<h2 id="rustdoc-include-works-with-anchors-too"><a class="header" href="#rustdoc-include-works-with-anchors-too">Rustdoc include works with anchors too</a></h2>
<pre><pre class="playground"><code class="language-rust"><span class="boring">fn some_other_function() {
</span><span class="boring">    println!("unused anchor");
</span><span class="boring">}
</span><span class="boring">
</span>fn main() {
    some_other_function();
}</code></pre></pre>
"##]]);
}
