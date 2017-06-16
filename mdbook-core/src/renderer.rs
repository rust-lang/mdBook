pub trait Plugin {
    // FIXME: Should these methods work with a Cow<str> instead?

    /// A function which is run immediately after loading a file from disk.
    ///
    /// This allows plugin creators to do any special preprocessing before it
    /// reaches the markdown parser (e.g. MathJax substitution). The plugin may
    /// or may not decide to make changes.
    fn preprocess_file(&mut self, _src: &str) -> Option<String> {
        None
    }

    /// The plugin function called just before a renderer writes the rendered
    /// file to disk.
    ///
    /// This is typically when you would go through and update links or add
    /// in a TOC.
    fn postprocess_file(&mut self, _src: &str) -> Option<String> {
        None
    }
}
