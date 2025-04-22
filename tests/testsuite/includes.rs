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
