use rust_coding_lib::listnode_lib::{ListNode, to_list};

struct Solution;
impl Solution {
    pub fn merge_two_lists(mut list1: Option<Box<ListNode>>, mut list2: Option<Box<ListNode>>) 
        -> Option<Box<ListNode>> {
        
        let mut dummy = Box::new(ListNode::new(0));
        let mut r_res = &mut dummy;
        while list1.is_some() && list2.is_some() {
            if list1.as_ref().unwrap().val < list2.as_ref().unwrap().val {
                let next = list1.as_mut().unwrap().next.take();
                r_res.next = list1;
                list1 = next;
            }
            else {
                let next = list2.as_mut().unwrap().next.take();
                r_res.next = list2;
                list2 = next;
            } 
            r_res = r_res.next.as_mut().unwrap();
        }
        r_res.next = if list1.is_some() {list1} else {list2};
        dummy.next
    }
}


fn main() {
    let list1 = to_list(vec![1,2,4]);
    let list2 = to_list(vec![1,3,4]);
    let expected = to_list(vec![1,1,2,3,4,4]);

    assert_eq!(Solution::merge_two_lists(list1, list2), expected);
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_two_empty_lists() {
        assert_eq!(Solution::merge_two_lists(None, None), None);
    }

    #[test]
    fn test_merge_one_empty_list() {
        let list = to_list(vec![1, 2, 3]);
        assert_eq!(Solution::merge_two_lists(None, to_list(vec![1, 2, 3])), list);
    }

    #[test]
    fn test_merge_two_non_empty_lists() {
        let list1 = to_list(vec![1, 2, 4]);
        let list2 = to_list(vec![1, 3, 4]);
        let expected = to_list(vec![1, 1, 2, 3, 4, 4]);
        assert_eq!(Solution::merge_two_lists(list1, list2), expected);
    }
}
