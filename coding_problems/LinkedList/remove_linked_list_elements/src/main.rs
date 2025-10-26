// 203. Remove Linked List Elements
use colored::*;
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


fn case_test(head: Option<Box<ListNode>>, val: i32, expected_new_head: Option<Box<ListNode>>) {
    let new_head = Solution::remove_elements(head.clone(), val);

    print!("{}", "before remove: ".yellow());
    print_list(head);
    if new_head == expected_new_head {
        print!("{}", "[Success] after remove: ".green());
        print_list(new_head);
    } else {
        print!("{}", "[Fail] after remove: ".red());
        print_list(new_head);
    }
}


fn main() {
    let head = to_list(vec![1, 2, 6, 3, 4, 5, 6]);
    let expected_head = to_list(vec![1, 2, 3, 4, 5]);
    let val = 6;
    case_test(head, val, expected_head);
    println!("{}", "---".blue());

    let head = to_list(vec![7,7,7,7]);
    let expected_head = None;
    let val = 7;
    case_test(head, val, expected_head);
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
        let expected = None;
        assert_eq!(new_head, expected);
    }
}