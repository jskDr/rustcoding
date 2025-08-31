pub mod treenode_lib;

#[cfg(test)]
mod tests {
    use super::treenode_lib::{to_tree, TreeWrapper};

    #[test]
    fn test_tree_visualization() {
        let tree = to_tree(vec![Some(4), Some(2), Some(7), Some(1), Some(3), Some(6), Some(9)]);
        
        // The custom Debug implementation for TreeWrapper will be used here.
        // The output will be a visual representation of the tree.
        println!("
Tree visualization:
{:?}", TreeWrapper(&tree));
    }
}