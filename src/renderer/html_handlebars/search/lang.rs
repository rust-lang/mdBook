use crate::renderer::html_handlebars::search::MAX_WORD_LENGTH_TO_INDEX;
use elasticlunr::lang::English;
use elasticlunr::Pipeline;
use once_cell::sync::OnceCell;
use regex::Regex;

pub struct Fallback {
    tokenize_regex: &'static Regex,
    english: &'static English,
}

impl Fallback {
    pub fn new() -> Self {
        static TOKENIZE_REGEX: OnceCell<Regex> = OnceCell::new();
        static ENGLISH: OnceCell<English> = OnceCell::new();
        Self {
            tokenize_regex: TOKENIZE_REGEX.get_or_init(|| Regex::new(
                r"[\p{Unified_Ideograph}\p{Hangul}]|[^\p{White_Space}\p{P}\p{Sm}\p{CurrencySymbol}\p{So}\p{Unified_Ideograph}\p{Hangul}\p{Z}\p{C}]+|\p{So}\p{Sk}?(\u200D\p{So}\p{Sk}?)*"
            ).unwrap()),
            english: ENGLISH.get_or_init(English::new),
        }
    }
}

impl elasticlunr::Language for Fallback {
    fn name(&self) -> String {
        "English, Chinese, Japanese, Korean, Vietnamese".into()
    }

    fn code(&self) -> String {
        "en".into()
    }

    fn tokenize(&self, text: &str) -> Vec<String> {
        self.tokenize_regex
            .find_iter(text)
            .map(|s| s.as_str())
            .filter(|s| s.len() <= MAX_WORD_LENGTH_TO_INDEX)
            .map(|s| s.to_lowercase())
            .collect()
    }

    fn make_pipeline(&self) -> Pipeline {
        let mut pipeline = self.english.make_pipeline();
        pipeline.queue.drain(0..2);
        pipeline
    }
}
