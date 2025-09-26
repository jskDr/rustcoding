use rust_coding_lib::listnode_lib::{ListNode, to_list};

struct Solution;
impl Solution {
    pub fn delete_duplicates(mut head: Option<Box<ListNode>>) -> Option<Box<ListNode>> {
        if head.is_none() {
            return head;
        }
        let mut node = head.as_mut().unwrap();
        while node.next.is_some() {
            if node.val == node.next.as_ref().unwrap().val {
                node.next = node.next.as_mut().unwrap().next.take();
            }
            else {
                node = node.next.as_mut().unwrap(); 
            }
        }
        head
    }
}

fn main() {
    let list = to_list(vec![1, 1, 2]);
    let expected = to_list(vec![1, 2]);

    assert_eq!(Solution::delete_duplicates(list), expected);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_duplicates_empty_list() {
        assert_eq!(Solution::delete_duplicates(None), None);

        let list = to_list(vec![1, 1, 2]);
        let expected = to_list(vec![1, 2]);
        assert_eq!(Solution::delete_duplicates(list), expected);

        let list = to_list(vec![1, 1, 2, 3, 3]);
        let expected = to_list(vec![1, 2, 3]);
        assert_eq!(Solution::delete_duplicates(list), expected);
    }
}

