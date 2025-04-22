//! Tests for the index preprocessor.

use crate::prelude::*;

// Checks basic README to index.html conversion.
#[test]
fn readme_to_index() {
    let mut test = BookTest::from_dir("index/basic_readme");
    test.check_main_file(
        "book/index.html",
        str![[r##"<h1 id="intro"><a class="header" href="#intro">Intro</a></h1>"##]],
    )
    .check_main_file(
        "book/first/index.html",
        str![[r##"<h1 id="first"><a class="header" href="#first">First</a></h1>"##]],
    )
    .check_main_file(
        "book/second/index.html",
        str![[r##"<h1 id="second"><a class="header" href="#second">Second</a></h1>"##]],
    )
    .check_toc_js(str![[r#"
<ol class="chapter">
<li class="chapter-item expanded affix ">
<a href="index.html">Intro</a>
</li>
<li class="chapter-item expanded ">
<a href="first/index.html">
<strong aria-hidden="true">1.</strong> First</a>
</li>
<li class="chapter-item expanded ">
<a href="second/index.html">
<strong aria-hidden="true">2.</strong> Second</a>
</li>
</ol>
"#]]);
    assert!(test.dir.join("book/index.html").exists());
    assert!(!test.dir.join("book/README.html").exists());
}
