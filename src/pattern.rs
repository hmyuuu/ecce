use regex::Regex;
use sha2::{Digest, Sha256};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct EccePattern {
    pub content: String,
    pub start_pos: usize,
    pub end_pos: usize,
    pub pattern_type: PatternType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PatternType {
    Inline,    // ecce ... ecce
    CodeBlock, // ```ecce ... ```
}

pub struct PatternDetector {
    processed_hashes: HashSet<String>,
}

impl PatternDetector {
    pub fn new() -> Self {
        Self {
            processed_hashes: HashSet::new(),
        }
    }

    /// Compute hash of pattern content to track what's been processed
    fn hash_content(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Check if a pattern has been processed
    pub fn is_processed(&self, content: &str) -> bool {
        let hash = Self::hash_content(content);
        self.processed_hashes.contains(&hash)
    }

    /// Mark a pattern as processed
    pub fn mark_processed(&mut self, content: &str) {
        let hash = Self::hash_content(content);
        self.processed_hashes.insert(hash);
    }

    /// Detect all ecce patterns in the given text
    pub fn detect_patterns(&self, text: &str) -> Vec<EccePattern> {
        let mut patterns = Vec::new();

        // Detect inline patterns: ecce ... ecce
        let inline_re = Regex::new(r"ecce\s+(.*?)\s+ecce").unwrap();
        for cap in inline_re.captures_iter(text) {
            let full_match = cap.get(0).unwrap();
            let content = cap.get(1).unwrap().as_str().to_string();

            if !self.is_processed(&content) {
                patterns.push(EccePattern {
                    content,
                    start_pos: full_match.start(),
                    end_pos: full_match.end(),
                    pattern_type: PatternType::Inline,
                });
            }
        }

        // Detect code block patterns: ```ecce ... ```
        let codeblock_re = Regex::new(r"```ecce\s*\n(.*?)\n```").unwrap();
        for cap in codeblock_re.captures_iter(text) {
            let full_match = cap.get(0).unwrap();
            let content = cap.get(1).unwrap().as_str().to_string();

            if !self.is_processed(&content) {
                patterns.push(EccePattern {
                    content,
                    start_pos: full_match.start(),
                    end_pos: full_match.end(),
                    pattern_type: PatternType::CodeBlock,
                });
            }
        }

        // Sort by position
        patterns.sort_by_key(|p| p.start_pos);
        patterns
    }

    /// Extract only new patterns from added text
    pub fn detect_new_patterns(&self, text: &str) -> Vec<EccePattern> {
        self.detect_patterns(text)
            .into_iter()
            .filter(|p| !self.is_processed(&p.content))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_pattern() {
        let detector = PatternDetector::new();
        let text = "Some text ecce what is apple? ecce more text";
        let patterns = detector.detect_patterns(text);

        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].content, "what is apple?");
        assert_eq!(patterns[0].pattern_type, PatternType::Inline);
    }

    #[test]
    fn test_codeblock_pattern() {
        let detector = PatternDetector::new();
        let text = "Some text\n```ecce\nwhat is apple?\n```\nmore text";
        let patterns = detector.detect_patterns(text);

        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].content, "what is apple?");
        assert_eq!(patterns[0].pattern_type, PatternType::CodeBlock);
    }

    #[test]
    fn test_multiple_patterns() {
        let detector = PatternDetector::new();
        let text = "ecce first question? ecce and ```ecce\nsecond question?\n```";
        let patterns = detector.detect_patterns(text);

        assert_eq!(patterns.len(), 2);
        assert_eq!(patterns[0].content, "first question?");
        assert_eq!(patterns[1].content, "second question?");
    }

    #[test]
    fn test_processed_tracking() {
        let mut detector = PatternDetector::new();
        let text = "ecce what is apple? ecce";

        let patterns = detector.detect_patterns(text);
        assert_eq!(patterns.len(), 1);

        detector.mark_processed(&patterns[0].content);

        let patterns_again = detector.detect_patterns(text);
        assert_eq!(patterns_again.len(), 0);
    }
}
