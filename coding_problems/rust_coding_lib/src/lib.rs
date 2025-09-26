pub mod treenode_lib;
pub mod listnode_lib;

#[cfg(test)]
mod tests {
    use super::treenode_lib::{to_tree, TreeWrapper};
    use super::listnode_lib::{to_list, ListNode};

    #[test]
    fn test_tree_visualization() {
        let tree = to_tree(vec![Some(4), Some(2), Some(7), Some(1), Some(3), Some(6), Some(9)]);
        
        // The custom Debug implementation for TreeWrapper will be used here.
        // The output will be a visual representation of the tree.
        println!("\nTree visualization:\n{:?}", TreeWrapper(&tree));
    }

    #[test]
    fn test_list_creation() {
        let list = to_list(vec![1, 2, 3]);
        let mut node1 = ListNode::new(1);
        let mut node2 = ListNode::new(2);
        let node3 = ListNode::new(3);
        node2.next = Some(Box::new(node3));
        node1.next = Some(Box::new(node2));
        assert_eq!(list, Some(Box::new(node1)));
    }
}