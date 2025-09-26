// 203. Remove Linked List Elements
use rust_coding_lib::listnode_lib::{ListNode, to_list, print_list};

struct Solution;

impl Solution {
    pub fn remove_elements(mut head: Option<Box<ListNode>>, val: i32) -> Option<Box<ListNode>> {
        let mut dummy = Box::new(ListNode::new(0));
        let mut cur = &mut dummy;

        while let Some(mut node) = head {
            let next = node.next.take();
            if node.val != val {
                cur.next = Some(node);
                cur = cur.next.as_mut().unwrap();
            }
            head = next;
        }
        dummy.next
    }
}


fn main() {
    let head = to_list(vec![1, 2, 6, 3, 4, 5, 6]);
    print!("before remove: ");
    print_list(head.clone());
    let new_head = Solution::remove_elements(head, 6);
    print!("after remove: ");
    print_list(new_head);

    let head = to_list(vec![7,7,7,7]);
    print!("before remove: ");
    print_list(head.clone());
    let new_head = Solution::remove_elements(head, 7);
    print!("after remove: ");
    print_list(new_head);
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_elements_1() {
        let head = to_list(vec![1, 2, 6, 3, 4, 5, 6]);
        let new_head = Solution::remove_elements(head, 6);
        let expected = to_list(vec![1, 2, 3, 4, 5]);
        assert_eq!(new_head, expected);
    }

    #[test]
    fn test_remove_elements_2() {
        let head = to_list(vec![7,7,7,7,7]);
        let new_head = Solution::remove_elements(head, 7);
        let expected = to_list(vec![]);
        assert_eq!(new_head, expected);
    }
}
