struct Solution;

impl Solution {
    pub fn third_max(nums: Vec<i32>) -> i32 {
        let mut h = vec![None;3];
        for n in nums {
            if h.contains(&Some(n)) {
                continue;
            }
            if h[0].is_none() || n > h[0].unwrap() {
                h[2] = h[1];
                h[1] = h[0];
                h[0] = Some(n);
            }
            else if h[1].is_none() || h[0].unwrap() > n && n > h[1].unwrap() {
                h[2] = h[1];
                h[1] = Some(n);
            }
            else if h[2].is_none() || h[1].unwrap() > n && n > h[2].unwrap() {
                h[2] = Some(n);
            }
        }
        match h[2] {
            Some(v) => v,
            None => h[0].unwrap()
        }
    }
}

fn main() {
    println!("{}", Solution::third_max(vec![1,2,3,4,5]));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        assert_eq!(1, Solution::third_max(vec![3,2,1]));
    }

    #[test]
    fn test2() {
        assert_eq!(2, Solution::third_max(vec![1,2]));
    }

    #[test]
    fn test3() {
        assert_eq!(1, Solution::third_max(vec![2,2,3,1]));
    }
}