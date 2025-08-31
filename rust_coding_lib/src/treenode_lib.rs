use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;

#[derive(PartialEq, Eq)]
pub struct TreeNode {
    pub val: i32,
    pub left: Option<Rc<RefCell<TreeNode>>>,
    pub right: Option<Rc<RefCell<TreeNode>>>,
}

impl TreeNode {
    #[inline]
    pub fn new(val: i32) -> Self {
        TreeNode {
            val,
            left: None,
            right: None,
        }
    }
}

// A wrapper for Option<Rc<RefCell<TreeNode>>> to provide a custom Debug implementation.
pub struct TreeWrapper<'a>(pub &'a Option<Rc<RefCell<TreeNode>>>);

impl<'a> fmt::Debug for TreeWrapper<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn print_node(f: &mut fmt::Formatter, node: &TreeNode, prefix: &str, is_left: bool) -> fmt::Result {
            if let Some(ref right) = node.right {
                print_node(f, &right.borrow(), &(prefix.to_owned() + if is_left { "│   " } else { "    " }), false)?;
            }
            writeln!(f, "{}{}{}", prefix, if is_left { "└── " } else { "┌── " }, node.val)?;
            if let Some(ref left) = node.left {
                print_node(f, &left.borrow(), &(prefix.to_owned() + if is_left { "    " } else { "│   " }), true)?;
            }
            Ok(())
        }

        if let Some(root_rc) = self.0 {
            let root_node = root_rc.borrow();
            if let Some(ref right) = root_node.right {
                print_node(f, &right.borrow(), "", false)?;
            }
            writeln!(f, "{}", root_node.val)?;
            if let Some(ref left) = root_node.left {
                print_node(f, &left.borrow(), "", true)?;
            }
        } else {
            writeln!(f, "(empty)")?;
        }
        Ok(())
    }
}


/// Creates a binary tree from a vector representation (level-order).
pub fn to_tree(nodes: Vec<Option<i32>>) -> Option<Rc<RefCell<TreeNode>>> {
    if nodes.is_empty() || nodes[0].is_none() {
        return None;
    }

    let root = Rc::new(RefCell::new(TreeNode::new(nodes[0].unwrap())));
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(root.clone());

    let mut i = 1;
    while !queue.is_empty() && i < nodes.len() {
        let node = queue.pop_front().unwrap();
        
        if i < nodes.len() {
            if let Some(val) = nodes[i] {
                let left_child = Rc::new(RefCell::new(TreeNode::new(val)));
                node.borrow_mut().left = Some(left_child.clone());
                queue.push_back(left_child);
            }
            i += 1;
        }

        if i < nodes.len() {
            if let Some(val) = nodes[i] {
                let right_child = Rc::new(RefCell::new(TreeNode::new(val)));
                node.borrow_mut().right = Some(right_child.clone());
                queue.push_back(right_child);
            }
            i += 1;
        }
    }
    Some(root)
}
