// 20. Valid Parentheses

use std::collections::{HashSet, HashMap};
struct Solution {}
impl Solution {
    pub fn is_valid(s: String) -> bool {
        let left:HashSet<_> = vec!['[', '{', '('].into_iter().collect();
        let right:HashMap<_,_> = vec![('[',']'), ('{','}'), ('(',')')].into_iter().collect();
        let mut stack = vec![];
        for a in s.chars() {
            if left.contains(&a) {
                stack.push(a);
            }
            else {
                if stack.last().map_or(false, |plast| right[plast] == a) {
                    stack.pop();
                }
                else {
                    return false;
                }
            }
        }
        stack.is_empty()
    }
}

fn main() {
    // Result: true
    println!("Result: {}", Solution::is_valid("()".to_string()));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_1() {
        assert_eq!(Solution::is_valid("()".to_string()), true);
    }

    #[test]
    fn test_2() {
        assert_eq!(Solution::is_valid("()[]{}".to_string()), true);
    }

    #[test]
    fn test_3() {
        assert_eq!(Solution::is_valid("(]".to_string()), false);
    }
}