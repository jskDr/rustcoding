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

struct Solution1;
use std::collections::BinaryHeap;
use std::collections::HashSet;
use std::cmp::Reverse;
impl Solution1 {
    pub fn third_max(nums: Vec<i32>) -> i32 {
        let mut h = BinaryHeap::new();
        let mut mx = i32::MIN;
        let nums: HashSet<_> = nums.into_iter().collect();

        for n in nums {
            if h.len() < 3 {
                h.push(Reverse(n));
            }
            else if h.peek().map_or(false, |v| v > &Reverse(n)) {
                h.pop();
                h.push(Reverse(n));
            } 
            mx = mx.max(n);
        }
        if h.len() < 3 {
            mx
        }
        else {
            h.peek().unwrap().0
        }
    }
}


fn main() {
    println!("{} == 3", Solution::third_max(vec![1,2,3,4,5]));
    println!("{} == 3", Solution1::third_max(vec![1,2,3,4,5]));
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