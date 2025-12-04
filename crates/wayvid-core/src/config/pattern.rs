//! Pattern matching utilities for output names
//!
//! Supports simple glob-style patterns:
//! - `*` matches any sequence of characters
//! - `?` matches any single character
//! - Exact matches without wildcards

/// Check if an output name matches a pattern
///
/// # Examples
/// ```
/// use wayvid_core::config::matches_pattern;
///
/// assert!(matches_pattern("HDMI-A-1", "HDMI-*"));
/// assert!(matches_pattern("HDMI-A-1", "HDMI-A-?"));
/// assert!(matches_pattern("HDMI-A-1", "HDMI-A-1"));
/// assert!(!matches_pattern("DP-1", "HDMI-*"));
/// ```
pub fn matches_pattern(name: &str, pattern: &str) -> bool {
    // No wildcards - exact match
    if !pattern.contains('*') && !pattern.contains('?') {
        return name == pattern;
    }

    // Convert pattern to regex-like matching
    match_glob(name, pattern)
}

/// Internal glob matching algorithm
fn match_glob(name: &str, pattern: &str) -> bool {
    let mut name_chars = name.chars().peekable();
    let mut pattern_chars = pattern.chars().peekable();

    while let Some(&p) = pattern_chars.peek() {
        match p {
            '*' => {
                pattern_chars.next();

                // * at end matches everything
                if pattern_chars.peek().is_none() {
                    return true;
                }

                // Try to match remaining pattern at each position
                let remaining_pattern: String = pattern_chars.clone().collect();
                for _ in 0..=name_chars.clone().count() {
                    let remaining_name: String = name_chars.clone().collect();
                    if match_glob(&remaining_name, &remaining_pattern) {
                        return true;
                    }
                    if name_chars.next().is_none() {
                        return false;
                    }
                }
                return false;
            }
            '?' => {
                pattern_chars.next();
                if name_chars.next().is_none() {
                    return false; // Pattern expects a char but name ended
                }
            }
            c => {
                pattern_chars.next();
                if name_chars.next() != Some(c) {
                    return false;
                }
            }
        }
    }

    // Both exhausted = match, otherwise no match
    name_chars.peek().is_none()
}

/// Find the best matching pattern for an output name
///
/// Returns the pattern that best matches, preferring:
/// 1. Exact matches
/// 2. Most specific patterns (fewest wildcards)
/// 3. Longest patterns
pub fn find_best_match<'a>(name: &str, patterns: &[&'a str]) -> Option<&'a str> {
    let mut candidates: Vec<(&str, usize)> = patterns
        .iter()
        .filter(|&&p| matches_pattern(name, p))
        .map(|&p| {
            // Score: exact=0, fewer wildcards & longer = lower score
            let wildcards = p.chars().filter(|&c| c == '*' || c == '?').count();
            let score = if p == name {
                0 // Exact match has highest priority
            } else {
                wildcards * 1000 - p.len() // Prefer fewer wildcards, then longer
            };
            (p, score)
        })
        .collect();

    if candidates.is_empty() {
        return None;
    }

    // Sort by score (lower is better)
    candidates.sort_by_key(|(_, score)| *score);
    Some(candidates[0].0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        assert!(matches_pattern("HDMI-A-1", "HDMI-A-1"));
        assert!(!matches_pattern("HDMI-A-1", "HDMI-A-2"));
    }

    #[test]
    fn test_star_wildcard() {
        assert!(matches_pattern("HDMI-A-1", "HDMI-*"));
        assert!(matches_pattern("HDMI-A-1", "*"));
        assert!(matches_pattern("HDMI-A-1", "HDMI-A-*"));
        assert!(matches_pattern("HDMI-A-1", "*-1"));
        assert!(!matches_pattern("DP-1", "HDMI-*"));
    }

    #[test]
    fn test_question_wildcard() {
        assert!(matches_pattern("HDMI-A-1", "HDMI-A-?"));
        assert!(matches_pattern("HDMI-A-1", "HDMI-?-1"));
        assert!(!matches_pattern("HDMI-A-11", "HDMI-A-?"));
    }

    #[test]
    fn test_mixed_wildcards() {
        assert!(matches_pattern("HDMI-A-1", "HDMI-*-?"));
        assert!(matches_pattern("HDMI-A-1", "*-?"));
        assert!(matches_pattern("HDMI-A-1", "?DMI-*"));
    }

    #[test]
    fn test_best_match() {
        let patterns = vec!["HDMI-*", "HDMI-A-*", "HDMI-A-1", "*"];

        // Exact match wins
        assert_eq!(find_best_match("HDMI-A-1", &patterns), Some("HDMI-A-1"));

        // Most specific pattern
        assert_eq!(find_best_match("HDMI-A-2", &patterns), Some("HDMI-A-*"));

        // Less specific
        assert_eq!(find_best_match("HDMI-B-1", &patterns), Some("HDMI-*"));

        // Fallback to catch-all
        assert_eq!(find_best_match("DP-1", &patterns), Some("*"));

        // No match
        assert_eq!(find_best_match("eDP-1", &["HDMI-*", "DP-*"]), None);
    }
}
