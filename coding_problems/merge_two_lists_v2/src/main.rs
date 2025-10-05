use rust_coding_lib::listnode_lib::{ListNode, to_list, print_list};

struct Solution {}
impl Solution {
    pub fn merge_two_lists(mut list1: Option<Box<ListNode>>, 
        mut list2: Option<Box<ListNode>>) -> Option<Box<ListNode>> {
        let mut org = Box::new(ListNode::new(0));
        let mut dummy = &mut org;
        while list1.is_some() && list2.is_some() { 
            if list1.as_ref().unwrap().val < list2.as_ref().unwrap().val {
                dummy.next = list1;
                dummy = dummy.next.as_mut().unwrap();
                list1 = dummy.next.take();
            }
            else {
                dummy.next = list2;
                dummy = dummy.next.as_mut().unwrap();
                list2 = dummy.next.take();
            }
        }
        if list1.is_none() {
            dummy.next = list2;
        }
        else {
            dummy.next = list1;
        }
        org.next
    }
}

fn main() {
    let list1 = to_list(vec![1,2,4]);
    let list2 = to_list(vec![1,3,4]);
    let res = Solution::merge_two_lists(list1, list2);
    print_list(res);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        let list1 = to_list(vec![1,2,4]);
        let list2 = to_list(vec![1,3,4]);
        let res = Solution::merge_two_lists(list1, list2);
        let expect = to_list(vec![1,1,2,3,4,4]);
        assert_eq!(res, expect);
    }

    #[test]
    fn test_2() {
        let list1 = to_list(vec![]);
        let list2 = to_list(vec![]);    
        let res = Solution::merge_two_lists(list1, list2);
        let expect = to_list(vec![]);
        assert_eq!(res, expect);
    }

    #[test]
    fn test_3() {
        let list1 = to_list(vec![]);
        let list2 = to_list(vec![0]);
        let res = Solution::merge_two_lists(list1, list2);
        let expect = to_list(vec![0]);
        assert_eq!(res, expect);
    }
}