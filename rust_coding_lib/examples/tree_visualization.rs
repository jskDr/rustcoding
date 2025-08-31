use rust_coding_lib::treenode_lib::{to_tree, TreeWrapper};

fn main() {
    println!("--- Example 1: A full binary tree ---");
    let tree1 = to_tree(vec![Some(4), Some(2), Some(7), Some(1), Some(3), Some(6), Some(9)]);
    println!("{:?}", TreeWrapper(&tree1));

    println!("\n--- Example 2: A left-leaning tree ---");
    let tree2 = to_tree(vec![Some(1), Some(2), None, Some(3), None, Some(4), None]);
    println!("{:?}", TreeWrapper(&tree2));
    
    println!("\n--- Example 3: A right-leaning tree ---");
    let tree3 = to_tree(vec![Some(1), None, Some(2), None, Some(3), None, Some(4)]);
    println!("{:?}", TreeWrapper(&tree3));

    println!("\n--- Example 4: An empty tree ---");
    let tree4 = to_tree(vec![]);
    println!("{:?}", TreeWrapper(&tree4));
    
    println!("\n--- Example 5: A single node tree ---");
    let tree5 = to_tree(vec![Some(1)]);
    println!("{:?}", TreeWrapper(&tree5));
}
