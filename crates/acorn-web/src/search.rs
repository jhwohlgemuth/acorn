//! Shared literal and regular expression matching for browser search fields.
use regex::{Regex, RegexBuilder};

/// A search pattern with explicit regex and case sensitivity options.
pub struct SearchPattern {
    query: String,
    folded: String,
    regex: Option<Regex>,
    case_sensitive: bool,
}
impl SearchPattern {
    /// Compile a search pattern, returning a readable error for invalid regular expressions.
    pub fn new(query: &str, regex: bool, case_sensitive: bool) -> Result<Self, String> {
        let query = if regex { query.to_owned() } else { query.trim().to_owned() };
        let compiled = if regex {
            RegexBuilder::new(&query)
                .case_insensitive(!case_sensitive)
                .build()
                .map(Some)
                .map_err(|error| error.to_string())
        } else {
            Ok(None)
        };
        compiled.map(|regex| Self {
            folded: query.to_lowercase(),
            query,
            regex,
            case_sensitive,
        })
    }
    /// Return whether the pattern matches a single text value.
    pub fn matches(&self, text: &str) -> bool {
        match &self.regex {
            | Some(regex) => regex.is_match(text),
            | None if self.case_sensitive => text.contains(&self.query),
            | None => text.to_lowercase().contains(&self.folded),
        }
    }
    /// Match every whitespace-separated literal term, or the full regular expression.
    pub fn matches_terms(&self, text: &str) -> bool {
        if self.regex.is_some() {
            self.matches(text)
        } else if self.case_sensitive {
            self.query.split_whitespace().all(|term| text.contains(term))
        } else {
            let folded_text = text.to_lowercase();
            self.folded.split_whitespace().all(|term| folded_text.contains(term))
        }
    }
    pub(crate) fn relevance_terms(&self) -> Vec<String> {
        if self.regex.is_some() {
            Vec::new()
        } else {
            let query = if self.case_sensitive { &self.query } else { &self.folded };
            query.split_whitespace().map(str::to_owned).collect()
        }
    }
    pub(crate) fn case_sensitive(&self) -> bool {
        self.case_sensitive
    }
}
