// The next line will cause a `testing` test to fail if the anchor feature is broken in such a way
// that the whole file gets mistakenly included.
assert!(!$TEST_STATUS);

// ANCHOR: myanchor
// ANCHOR: unendinganchor
// The next line will cause a `rendered_output` test to fail if the anchor feature is broken in
// such a way that the content between anchors isn't included.
// unique-string-for-anchor-test
assert!($TEST_STATUS);
// ANCHOR_END: myanchor

pub mod indent {
    // ANCHOR: indentedanchor
    pub fn indented_function() {
        // This extra indent remains
    }
    // ANCHOR_END: indentedanchor
}
