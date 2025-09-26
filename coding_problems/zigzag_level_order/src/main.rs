// [Problem link] https://leetcode.com/problems/binary-tree-zigzag-level-order-traversal
use std::collections::VecDeque;
use std::cell::RefCell;
use std::rc::Rc;
use rust_coding_lib::treenode_lib::{TreeNode, to_tree};

struct Solution;

impl Solution {
    pub fn zigzag_level_order(root: Option<Rc<RefCell<TreeNode>>>) -> Vec<Vec<i32>> {
        let mut res = vec![];
        if root.is_none() {
            return res;
        }
        
        let mut q = VecDeque::new();
        q.push_back(root.unwrap());
        let mut lr_flag = true;
        
        while !q.is_empty() {
            let lq = q.len();
            let mut level = Vec::new();
            
            for _ in 0..lq {
                let node_rc = q.pop_front().unwrap();
                let node = node_rc.borrow();
                level.push(node.val);
                
                if let Some(left) = node.left.clone() {
                    q.push_back(left);
                }
                if let Some(right) = node.right.clone() {
                    q.push_back(right);
                }
            }
            
            if !lr_flag {
                level.reverse();
            }
            res.push(level);
            lr_flag = !lr_flag;
        }
        
        res
    }
}


fn main() {
    let root = to_tree(vec![Some(3), Some(9), Some(20), None, None, Some(15), Some(7)]);
    let result = Solution::zigzag_level_order(root);
    let expected = vec![
        vec![3],
        vec![20, 9],
        vec![15, 7],
    ];
    assert_eq!(result, expected);
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zigzag_level_order() {
        // Build the following tree:
        //       3
        //      / \
        //     9  20
        //        / \
        //       15  7
        let root = to_tree(vec![Some(3), Some(9), Some(20), None, None, Some(15), Some(7)]);
        let result = Solution::zigzag_level_order(root);
        let expected = vec![
            vec![3],
            vec![20, 9],
            vec![15, 7],
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_empty_tree() {
        let result = Solution::zigzag_level_order(None);
        let expected: Vec<Vec<i32>> = vec![];
        assert_eq!(result, expected);
    }
}