// Definition for singly-linked list.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ListNode {
  pub val: i32,
  pub next: Option<Box<ListNode>>
}


impl ListNode {
  #[inline]
  pub fn new(val: i32) -> Self {
    ListNode {
      next: None,
      val
    }
  }
}


// Helper function to create a linked list from a vector
pub fn to_list(vec: Vec<i32>) -> Option<Box<ListNode>> {
    let mut current = None;
    for &val in vec.iter().rev() {
        let mut new_node = ListNode::new(val);
        new_node.next = current;
        current = Some(Box::new(new_node));
    }
    current
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        assert_eq!(ListNode::new(1), ListNode { val: 1, next: None });
    }

    #[test]
    fn test_to_list() {
        let list = to_list(vec![1, 2, 3]);

        let mut node1 = ListNode::new(1);
        let mut node2 = ListNode::new(2);
        let node3 = ListNode::new(3);
        node2.next = Some(Box::new(node3));
        node1.next = Some(Box::new(node2));

        assert_eq!(list, Some(Box::new(node1)));
    }

    #[test]
    fn test_to_list_single() {
        let list = to_list(vec![1]);
        assert_eq!(list, Some(Box::new(ListNode::new(1))));
    }

    #[test]
    fn test_to_list_empty() {
        let list = to_list(vec![]);
        assert_eq!(list, None);
    }
}
