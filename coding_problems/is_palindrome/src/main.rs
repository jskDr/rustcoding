struct Solution;

impl Solution {
    pub fn is_palindrome(x: i32) -> bool {
        let s: Vec<char> = x.to_string().chars().collect();
        for i in 0..s.len() {
            if s[i] != s[s.len()-1-i] {
                return false;
            }
        }
        true
    }
}

fn main() {
    let res = Solution::is_palindrome(121);
    println!("is_palindrome(121) -> {}", res);
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_is_palindrome() {
        assert_eq!(Solution::is_palindrome(121), true);
        assert_eq!(Solution::is_palindrome(-121), false);
        assert_eq!(Solution::is_palindrome(10), false);
    }
}