// [Problem link] https://leetcode.com/problems/binary-tree-zigzag-level-order-traversal
// src/lib.rs
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::VecDeque;

#[derive(Debug, PartialEq, Eq)]
pub struct TreeNode {
    pub val: i32,
    pub left: Option<Rc<RefCell<TreeNode>>>,
    pub right: Option<Rc<RefCell<TreeNode>>>,
}

impl TreeNode {
    #[inline]
    pub fn new(val: i32) -> Rc<RefCell<TreeNode>> {
        Rc::new(RefCell::new(TreeNode {
            val,
            left: None,
            right: None,
        }))
    }
}

pub struct Solution;

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
        
        let root = TreeNode::new(3);
        let left = TreeNode::new(9);
        let right = TreeNode::new(20);
        let right_left = TreeNode::new(15);
        let right_right = TreeNode::new(7);

        // Connect nodes
        right.borrow_mut().left = Some(right_left);
        right.borrow_mut().right = Some(right_right);
        
        root.borrow_mut().left = Some(left);
        root.borrow_mut().right = Some(right);

        let result = Solution::zigzag_level_order(Some(root));
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