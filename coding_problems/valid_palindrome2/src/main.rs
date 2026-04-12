struct Solution;

impl Solution {
    /// Checks if one deletion can make `s` a palindrome.
    pub fn valid_palindrome(s: String) -> bool {
        if s.is_empty() {
            return true;
        }

        /// Validates the current substring.
        fn helper(mut st: usize, mut ed: usize, s: &[u8], failed: bool) -> bool {
            while st < ed { 
                if st < ed {
                    if s[st] != s[ed]{
                        if failed {
                            return false;
                        }
                        else {
                            // On the first mismatch, try skipping either side once.
                            return helper(st+1, ed, s, true) || 
                                helper(st, ed-1, s, true);
                        }
                    }
                    st += 1;
                    ed -= 1;
                }
            }
            true
        }
        let s = s.as_bytes();
        helper(0, s.len()-1, &s, false)
    }
}

/// Runs one sample check.
fn _main_sync(task_idx: usize) -> bool {
    println!("Task index: {}", task_idx);
    println!("Testing code for valid_palindrome2. Fpr more testing, please use: cargo test");
    let s = "abbca".to_string();
    println!("We are testing now: {}", s);
    let result = Solution::valid_palindrome(s);
    println!("Result: {} == true", result);
    result
}

/// Runs sample tasks concurrently.
#[tokio::main]
async fn main() {
    let task1 = tokio::task::spawn(async {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        _main_sync(1)
    });
    let task2 = tokio::task::spawn(async {
        _main_sync(2)
    });

    let result1 = task1.await.unwrap();
    let result2 = task2.await.unwrap();
    println!("Task1 returned: {}", result1);
    println!("Task2 returned: {}", result2);
}

#[cfg(test)]
mod tests {
    use super::Solution;

    /// Tests a valid palindrome.
    #[test]
    fn test_1() {
        let s = "abba".to_string();
        assert_eq!(Solution::valid_palindrome(s), true);
    }

    /// Tests an invalid palindrome.
    #[test]
    fn test_2() {
        let s = "abc".to_string();
        assert_eq!(Solution::valid_palindrome(s), false);
    }

    /// Tests one-removal success.
    #[test]
    fn test_3() {
        let s = "abbca".to_string();
        assert_eq!(Solution::valid_palindrome(s), true);
    }
}
