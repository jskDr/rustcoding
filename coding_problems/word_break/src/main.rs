//! Solves the Word Break problem using depth-first search with memoization.
//!
//! Failed starting indexes are cached to avoid repeating the same work.

use std::collections::HashSet;

/// Solver for the Word Break problem.
///
/// Determines whether a string can be split into dictionary words.
struct Solution;

impl Solution {
    /// Returns `true` when `s` can be segmented into words from `word_dict`.
    ///
    /// Uses recursive search with memoized failed indexes.
    ///
    /// # Function Prototype
    ///
    /// ```rust
    /// pub fn word_break(s: String, word_dict: Vec<String>) -> bool
    /// ```
    ///
    /// `s`: input string to segment.
    ///
    /// `word_dict`: list of allowed words.
    ///
    /// Returns `true` if `s` can be fully segmented, otherwise `false`.
    pub fn word_break(s: String, word_dict: Vec<String>) -> bool {
        fn test_word(i: usize, word: &String, s: &String) -> bool {
            if i + word.len() > s.len() {return false;}
            word[..] == s[i..i+word.len()]
        }
        fn check(i: usize, fail_points:&mut HashSet<usize>, 
            s: &String, word_dict: &Vec<String>) -> bool {
            if i == s.len() {return true;}
            if fail_points.contains(&i) {
                return false;
            }
            for word in word_dict.iter() {
                if test_word(i, word, s) {
                    if check(i+word.len(), fail_points, s, word_dict) {
                        return true;
                    }
                }
            }
            fail_points.insert(i);
            false
        }
        let mut fail_points = HashSet::new();
        check(0, &mut fail_points, &s, &word_dict)
    }
}

fn main() {
    let s = "leetcode".to_string();
    let word_dict = vec!["leet".to_string(), "code".to_string()];
    assert_eq!(Solution::word_break(s, word_dict), true);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_word_break_1() {
        let s = "leetcode".to_string();
        let word_dict = vec!["leet".to_string(), "code".to_string()];
        assert_eq!(Solution::word_break(s, word_dict), true);
    }

    #[test]
    fn test_word_break_2() {
        let s = "applepenapple".to_string();
        let word_dict = vec!["apple".to_string(), "pen".to_string()];
        assert_eq!(Solution::word_break(s, word_dict), true);
    }

    #[test]
    fn test_word_break_3() {
        let s = "catsandog".to_string();
        let word_dict = vec![
            "cats".to_string(),
            "dog".to_string(),
            "sand".to_string(),
            "and".to_string(),
            "cat".to_string(),
        ];
        assert_eq!(Solution::word_break(s, word_dict), false);
    }
}
