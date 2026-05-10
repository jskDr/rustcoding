# Valid Palindrome II 코드 리뷰
[[Valid Palindrome II]]

```Rust
impl Solution {
    /// Checks if one deletion can make `s` a palindrome. (second solution)
    pub fn valid_palindrome(s: String) -> bool {
        let s:Vec<_> = s.chars().collect();
        let is_pal = |l: usize, r: usize| s[l..=r].iter().eq(s[l..=r].iter().rev());
        let (mut st, mut ed) = (0, s.len()-1);
        while st < ed {
            if s[st] != s[ed] {
                return is_pal(st + 1, ed) || is_pal(st, ed - 1);
            }
            st += 1;
            ed -= 1;
        }
        true
    }
}
```
