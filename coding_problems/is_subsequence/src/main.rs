use colored::*;

fn is_subsequence(s: String, t: String) -> bool {
    let mut tstream = t.chars();
    s.chars().all(|sc| tstream.any(|tc| tc == sc))
}


fn main() {
    println!("Coding Problem: Is Subsequence");
    println!("Testing: is_subsequence(\"abc\".to_string(), \"a_x_b_y_c\".to_string()");
    if is_subsequence("abc".to_string(), "a_x_b_y_c".to_string()) {
        println!("{}", "...Passed".blue())
    } else {
        println!("{}", "...Failed".red())
    }    
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_subsequence() {
        let cases = vec![
            ("abc", "a_x_b_y_c", true),  // in order
            ("abc", "acb", false),       // wrong order
            ("", "anything", true),      // empty s is always subsequence
            ("abc", "", false),          // s not empty, t empty
            ("abc", "abc", true),        // identical
            ("aaaa", "aa", false),       // too few repeats
            ("ace", "abcde", true),      // spaced subsequence
            ("aec", "abcde", false),     // fails at order
        ];

        for (s, t, expected) in cases {
            assert_eq!(
                is_subsequence(s.to_string(), t.to_string()),
                expected,
                "Failed on s={:?}, t={:?}",
                s,
                t
            );
        }
    }
}
