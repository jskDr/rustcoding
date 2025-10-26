// 3217. Delete Nodes From Linked List Present in Array

use std::collections::HashSet;
use rust_coding_lib::listnode_lib::{ListNode,to_list,print_list};

struct Solution;
impl Solution {
    pub fn modified_list(nums: Vec<i32>, mut head: Option<Box<ListNode>>) -> Option<Box<ListNode>> {
        let s = nums.into_iter().collect::<HashSet<_>>();
        let mut v = Vec::new();
        while let Some(mut node) = head {
            head = node.next.take();
            if !s.contains(&node.val) {
                v.push(node);
            }
        }
        // println!("{:?}", v);
        let mut head = None;
        while let Some(mut node) = v.pop() {
            node.next = head;
            head = Some(node);
        }
        head
    }
}

fn main() {
    let nums = vec![1, 2, 3];
    let head = to_list(vec![1, 2, 3, 4, 5]);
    let result = Solution::modified_list(nums, head);
    print_list(result);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        let nums = vec![1, 2, 3];
        let head = to_list(vec![1, 2, 3, 4, 5]);
        let result = Solution::modified_list(nums, head);
        assert_eq!(to_list(vec![4, 5]), result);
    }

    #[test]
    fn test_2() {
        let nums = vec![1];
        let head = to_list(vec![1, 2, 1, 2, 1, 2]);
        let result = Solution::modified_list(nums, head);
        assert_eq!(to_list(vec![2, 2, 2]), result);
    }

    #[test]
    fn test_3() {
        let nums = vec![5];
        let head = to_list(vec![1, 2, 3, 4]);
        let result = Solution::modified_list(nums, head);
        assert_eq!(to_list(vec![1, 2, 3, 4]), result);
    }
}