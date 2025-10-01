// 496. Next Greater Element I
use std::collections::HashMap;

struct Solution;

impl Solution {
    pub fn next_greater_element(nums1: Vec<i32>, nums2: Vec<i32>) -> Vec<i32> {
        let mut map = HashMap::new();
        let mut stack = Vec::new();
        for n in nums2 {
            while let Some(&top) = stack.last() {
                if n > top {
                    map.insert(stack.pop().unwrap(), n);
                } else {
                    break;
                }
            }
            stack.push(n);
        }

        nums1.iter().map(|&n| *map.get(&n).unwrap_or(&-1)).collect()
    }
}

fn main() {
    let nums1 = vec![4, 1, 2];
    let nums2 = vec![1, 3, 4, 2];
    let result = Solution::next_greater_element(nums1, nums2);
    println!("Result: {:?}", result);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_greater_element() {
        // Test case 1
        let nums1 = vec![4, 1, 2];
        let nums2 = vec![1, 3, 4, 2];
        let expected = vec![-1, 3, -1];
        let result = Solution::next_greater_element(nums1, nums2);
        assert_eq!(expected, result);

        // Test case 2
        let nums1 = vec![2, 4];
        let nums2 = vec![1, 2, 3, 4];
        let expected = vec![3, -1];
        let result = Solution::next_greater_element(nums1, nums2);
        assert_eq!(expected, result);

        // Test case 3
        let nums1 = vec![1, 3, 5, 2, 4];
        let nums2 = vec![6, 5, 4, 3, 2, 1, 7];
        let expected = vec![7, 7, 7, 7, 7];
        let result = Solution::next_greater_element(nums1, nums2);
        assert_eq!(expected, result);

        // Test case 4: No next greater element
        let nums1 = vec![4, 3, 2, 1];
        let nums2 = vec![4, 3, 2, 1];
        let expected = vec![-1, -1, -1, -1];
        let result = Solution::next_greater_element(nums1, nums2);
        assert_eq!(expected, result);

        // Test case 5: Empty nums1
        let nums1 = vec![];
        let nums2 = vec![1, 2, 3, 4];
        let expected: Vec<i32> = vec![];
        let result = Solution::next_greater_element(nums1, nums2);
        assert_eq!(expected, result);
    }
}