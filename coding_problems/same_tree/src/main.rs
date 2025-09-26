use rust_coding_lib::treenode_lib::{TreeNode, TreeWrapper};
use std::rc::Rc;
use std::cell::RefCell;

pub struct Solution;

impl Solution {
    pub fn is_same_tree(p: Option<Rc<RefCell<TreeNode>>>, q: Option<Rc<RefCell<TreeNode>>>) -> bool {
        if p.is_none() && q.is_none() {
            return true;
        }
        if p.is_none() || q.is_none() {
            return false;
        }
        let p = p.unwrap();
        let q = q.unwrap();
        p.borrow().val == q.borrow().val &&
            Solution::is_same_tree(p.borrow().left.clone(), q.borrow().left.clone()) &&
            Solution::is_same_tree(p.borrow().right.clone(), q.borrow().right.clone()) 
    }
    pub fn is_same_tree_r2(p: Option<Rc<RefCell<TreeNode>>>, q: Option<Rc<RefCell<TreeNode>>>) -> bool {
        if p.is_none() && q.is_none() {
            return true;
        }
        if p.is_none() || q.is_none() {
            return false;
        }
        if let (Some(p), Some(q)) = (p, q) {
            let p = p.borrow();
            let q = q.borrow();
            p.val == q.val &&
                Solution::is_same_tree(p.left.clone(), q.left.clone()) &&
                Solution::is_same_tree(p.right.clone(), q.right.clone()) 
        }
        else {
            false
        }
    }
    pub fn is_same_tree_r1(p: Option<Rc<RefCell<TreeNode>>>, q: Option<Rc<RefCell<TreeNode>>>) -> bool {
        match (p, q) {
            (None, None) => true,
            (Some(p_node), Some(q_node)) => {
                let p_borrow = p_node.borrow();
                let q_borrow = q_node.borrow();
                p_borrow.val == q_borrow.val &&
                Solution::is_same_tree(p_borrow.left.clone(), q_borrow.left.clone()) &&
                Solution::is_same_tree(p_borrow.right.clone(), q_borrow.right.clone())
            },
            _ => false,
        }
    }
}

fn main() {
    // Example Usage:
    // Tree 1:
    //   1
    //  / \
    // 2   3
    let p = Some(Rc::new(RefCell::new(TreeNode {
        val: 1,
        left: Some(Rc::new(RefCell::new(TreeNode::new(2)))),
        right: Some(Rc::new(RefCell::new(TreeNode::new(3)))),
    })));

    // Tree 2:
    //   1
    //  / \
    // 2   3
    let q = Some(Rc::new(RefCell::new(TreeNode {
        val: 1,
        left: Some(Rc::new(RefCell::new(TreeNode::new(2)))),
        right: Some(Rc::new(RefCell::new(TreeNode::new(3)))),
    })));
    println!("Tree 1: \n{:#?}", TreeWrapper(&p));
    println!("Tree 2: \n{:#?}", TreeWrapper(&q));
    println!("Are the trees the same? {}", Solution::is_same_tree(p, q)); // Expected: true
}


#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a new node
    fn new_node(val: i32, left: Option<Rc<RefCell<TreeNode>>>, right: Option<Rc<RefCell<TreeNode>>>) -> Option<Rc<RefCell<TreeNode>>> {
        Some(Rc::new(RefCell::new(TreeNode{val, left, right})))
    }

    fn new_leaf(val: i32) -> Option<Rc<RefCell<TreeNode>>> {
        Some(Rc::new(RefCell::new(TreeNode::new(val))))
    }

    #[test]
    fn test_same_trees() {
        let p = new_node(1, new_leaf(2), new_leaf(3));
        let q = new_node(1, new_leaf(2), new_leaf(3));
        assert!(Solution::is_same_tree(p, q));
    }

    #[test]
    fn test_different_structure() {
        let p = new_node(1, new_leaf(2), None);
        let q = new_node(1, None, new_leaf(2));
        assert!(!Solution::is_same_tree(p, q));
    }

    #[test]
    fn test_different_values() {
        let p = new_node(1, new_leaf(2), new_leaf(3));
        let q = new_node(1, new_leaf(2), new_leaf(4));
        assert!(!Solution::is_same_tree(p, q));
    }

    #[test]
    fn test_one_empty() {
        let p = new_node(1, None, None);
        let q = None;
        assert!(!Solution::is_same_tree(p, q));
    }

    #[test]
    fn test_other_one_empty() {
        let p = None;
        let q = new_node(1, None, None);
        assert!(!Solution::is_same_tree(p, q));
    }

    #[test]
    fn test_both_empty() {
        let p = None;
        let q = None;
        assert!(Solution::is_same_tree(p, q));
    }

    #[test]
    fn test_complex_same_trees() {
        let p = new_node(5, new_node(3, new_leaf(1), new_leaf(4)), new_node(8, new_leaf(7), new_leaf(9)));
        let q = new_node(5, new_node(3, new_leaf(1), new_leaf(4)), new_node(8, new_leaf(7), new_leaf(9)));
        assert!(Solution::is_same_tree(p, q));
    }

    #[test]
    fn test_complex_different_trees() {
        let p = new_node(5, new_node(3, new_leaf(1), new_leaf(4)), new_node(8, new_leaf(7), new_leaf(9)));
        let q = new_node(5, new_node(3, new_leaf(1), new_leaf(4)), new_node(8, new_leaf(7), new_leaf(10)));
        assert!(!Solution::is_same_tree(p, q));
    }
}