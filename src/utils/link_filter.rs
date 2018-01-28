use url::Url;
use relative_path::RelativePath;

/// Translate the given destination from a relative link with an '.md' extension, to a link with
/// a '.html' extension.
pub struct ChangeExtLinkFilter<'a, F> {
    base: &'a RelativePath,
    is_dest: F,
    expected: &'a str,
    ext: &'a str,
}

impl<'a, F> ChangeExtLinkFilter<'a, F>
    where F: Fn(&RelativePath) -> bool
{
    pub fn new(base: &'a RelativePath, is_dest: F, expected: &'a str, ext: &'a str) -> ChangeExtLinkFilter<'a, F> {
        ChangeExtLinkFilter {
            base: base,
            is_dest: is_dest,
            expected: expected,
            ext: ext,
        }
    }
}

impl<'a, F> LinkFilter for ChangeExtLinkFilter<'a, F>
    where F: Fn(&RelativePath) -> bool
{
    fn apply(&self, dest: &str) -> Option<String> {
        use url::ParseError;

        // Verify that specified URL is relative.
        if let Err(ParseError::RelativeUrlWithoutBase) = Url::parse(dest) {
            // extract fragment.
            let mut split = dest.splitn(2, '#');

            if let Some(base) = split.next() {
                let dest = RelativePath::new(base);

                if Some(self.expected) == dest.extension() && (self.is_dest)(dest) {
                    let dest = self.base.join_normalized(dest).with_extension(self.ext);
                    let dest = dest.display().to_string();

                    if let Some(fragment) = split.next() {
                        return Some(format!("{}#{}", dest, fragment));
                    }

                    return Some(dest);
                }
            }
        }

        None
    }
}

/// A filter to optionally apply to links.
pub trait LinkFilter {
    /// Optionally translate the given destination, if applicable.
    fn apply(&self, dest: &str) -> Option<String>;
}
