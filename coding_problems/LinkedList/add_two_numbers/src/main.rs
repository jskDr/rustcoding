use rust_coding_lib::listnode_lib::{ListNode, print_list, to_list};
struct Solution;
impl Solution {
    pub fn add_two_numbers(mut l1: Option<Box<ListNode>>, mut l2: Option<Box<ListNode>>) -> Option<Box<ListNode>> {
        let mut dummy = Box::new(ListNode::new(0));
        let mut curr = &mut dummy;
        let mut carry = 0;
        while l1.is_some() || l2.is_some() || carry != 0 {
            let v1 = match l1 {
                Some(x) => {l1 = x.next; x.val},
                None => 0
            };
            let v2 = match l2 {
                Some(x) => {l2 = x.next; x.val},
                None => 0
            };
            let v = v1 + v2 + carry;
            carry = v / 10;
            curr.next = Some(Box::new(ListNode::new(v % 10)));
            curr = curr.next.as_mut().unwrap();
        }
        dummy.next
    }
}

fn main() {
    let l1 = to_list(vec![1, 2, 3]);
    let l2 = to_list(vec![9, 8, 7]);
    let l3 = Solution::add_two_numbers(l1, l2);
    print_list(l3);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        let l1 = to_list(vec![2, 4, 3]);
        let l2 = to_list(vec![5, 6, 4]);
        let result = Solution::add_two_numbers(l1, l2);
        let expected = to_list(vec![7, 0, 8]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_2() {
        let l1 = to_list(vec![0]);
        let l2 = to_list(vec![0]);
        let result = Solution::add_two_numbers(l1, l2);
        let expected = to_list(vec![0]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_3() {
        let l1 = to_list(vec![9, 9, 9, 9, 9, 9, 9]);
        let l2 = to_list(vec![9, 9, 9, 9]);
        let result = Solution::add_two_numbers(l1, l2);
        let expected = to_list(vec![8, 9, 9, 9, 0, 0, 0, 1]);
        assert_eq!(result, expected);
    }
}