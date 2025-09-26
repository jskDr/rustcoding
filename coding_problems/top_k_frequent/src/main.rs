// 347. Top K Frequent Elements
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Reverse;

pub struct Solution;

impl Solution {
    pub fn top_k_frequent(nums: Vec<i32>, k: i32) -> Vec<i32> {
        let mut bh: BinaryHeap<Reverse<(i32, i32)>> = 
            vec![Reverse((0, 0)); k as usize].into_iter().collect();
        let mut map = HashMap::new();
        for n in nums {
            *map.entry(n).or_insert(0) += 1;
        }
        for (n, c) in map {
            bh.push(Reverse((c, n)));
            bh.pop();  // Keep heap at size k
        }
        let mut res = vec![];
        // while let Some(Reverse((_, n))) = bh.pop() {
        while let Some(r) = bh.pop() {
            res.push(r.0.1);
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_1() {
        let nums = vec![1,1,1,2,2,3];
        let k = 2;
        let mut result = Solution::top_k_frequent(nums, k);
        result.sort();
        assert_eq!(result, vec![1, 2]);
    }

    #[test]
    fn test_case_2() {
        let nums = vec![1];
        let k = 1;
        let mut result = Solution::top_k_frequent(nums, k);
        result.sort();
        assert_eq!(result, vec![1]);
    }

    #[test]
    fn test_case_3() {
        let nums = vec![4,1,-1,2,-1,2,3];
        let k = 2;
        let mut result = Solution::top_k_frequent(nums, k);
        result.sort();
        assert_eq!(result, vec![-1, 2]);
    }

    #[test]
    fn test_case_with_same_frequency() {
        let nums = vec![1, 2, 3, 4];
        let k = 2;
        let result = Solution::top_k_frequent(nums, k);
        // All appear once, any 2 of them is acceptable
        assert_eq!(result.len(), 2);
        for e in result {
            assert!(vec![1,2,3,4].contains(&e));
        }
    }
}
