use rust_coding_lib::treenode_lib::{to_tree, TreeNode, TreeWrapper};
use std::cell::RefCell;
use std::rc::Rc;

pub struct Solution;

impl Solution {
    pub fn invert_tree(root: Option<Rc<RefCell<TreeNode>>>) -> Option<Rc<RefCell<TreeNode>>> {
        if let Some(root) = root {
            let r1 = root.borrow();
            let mut r2 = TreeNode::new(r1.val);
            r2.left = Solution::invert_tree(r1.right.clone());
            r2.right = Solution::invert_tree(r1.left.clone());
            Some(Rc::new(RefCell::new(r2)))
        } else {
            None
        }
    }
}

fn main() {
    // Example Usage:
    // Tree:
    //   4
    //  / \
    // 2   7
    //    / \
    //   6   9
    let root = to_tree(vec![
        Some(4),
        Some(2),
        Some(7),
        None,
        None,
        Some(6),
        Some(9),
    ]);

    println!("Original tree: \n{:#?}", TreeWrapper(&root));

    let inverted_tree = Solution::invert_tree(root);

    println!("\nInverted tree: \n{:#?}", TreeWrapper(&inverted_tree));
}


#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a new node
    fn new_node(
        val: i32,
        left: Option<Rc<RefCell<TreeNode>>>,
        right: Option<Rc<RefCell<TreeNode>>>,
    ) -> Option<Rc<RefCell<TreeNode>>> {
        Some(Rc::new(RefCell::new(TreeNode { val, left, right })))
    }

    fn new_leaf(val: i32) -> Option<Rc<RefCell<TreeNode>>> {
        Some(Rc::new(RefCell::new(TreeNode::new(val))))
    }

    // Helper to check if two trees are the same.
    fn is_same_tree(p: Option<Rc<RefCell<TreeNode>>>, q: Option<Rc<RefCell<TreeNode>>>) -> bool {
        match (p, q) {
            (None, None) => true,
            (Some(p_node), Some(q_node)) => {
                let p_borrow = p_node.borrow();
                let q_borrow = q_node.borrow();
                p_borrow.val == q_borrow.val
                    && is_same_tree(p_borrow.left.clone(), q_borrow.left.clone())
                    && is_same_tree(p_borrow.right.clone(), q_borrow.right.clone())
            }
            _ => false,
        }
    }

    #[test]
    fn test_empty_tree() {
        let root = None;
        let inverted = Solution::invert_tree(root);
        let expected = None;
        assert!(is_same_tree(inverted, expected));
    }

    #[test]
    fn test_single_node_tree() {
        let root = new_leaf(1);
        let inverted = Solution::invert_tree(root);
        let expected = new_leaf(1);
        assert!(is_same_tree(inverted, expected));
    }

    #[test]
    fn test_simple_tree() {
        //   4
        //  / \
        // 2   7
        //    / \
        //   6   9
        let root = new_node(4, new_leaf(2), new_node(7, new_leaf(6), new_leaf(9)));

        // Expected:
        //   4
        //  / \
        // 7   2
        // / \
        //9   6
        let expected = new_node(4, new_node(7, new_leaf(9), new_leaf(6)), new_leaf(2));

        let inverted = Solution::invert_tree(root);
        assert!(is_same_tree(inverted, expected));
    }

    #[test]
    fn test_another_tree() {
        //    2
        //   / \
        //  1   3
        let root = new_node(2, new_leaf(1), new_leaf(3));

        // Expected:
        //    2
        //   / \
        //  3   1
        let expected = new_node(2, new_leaf(3), new_leaf(1));

        let inverted = Solution::invert_tree(root);
        assert!(is_same_tree(inverted, expected));
    }

    #[test]
    fn test_full_tree() {
        //      4
        //    /   \
        //   2     7
        //  / \   / \
        // 1   3 6   9
        let root = new_node(
            4,
            new_node(2, new_leaf(1), new_leaf(3)),
            new_node(7, new_leaf(6), new_leaf(9)),
        );

        // Expected
        //      4
        //    /   \
        //   7     2
        //  / \   / \
        // 9   6 3   1
        let expected = new_node(
            4,
            new_node(7, new_leaf(9), new_leaf(6)),
            new_node(2, new_leaf(3), new_leaf(1)),
        );

        let inverted = Solution::invert_tree(root);
        assert!(is_same_tree(inverted, expected));
    }
}