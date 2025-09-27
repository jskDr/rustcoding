// 2487. Remove Nodes From Linked List
use colored::*;
use rust_coding_lib::listnode_lib::{ListNode, to_list, print_list};

struct Solution;
impl Solution {
    fn remove_nodes(mut head: Option<Box<ListNode>>) -> Option<Box<ListNode>>{
        let mut stack: Vec<Box<ListNode>> = vec![];
        while let Some(mut node) = head {
            head = node.next.take();
            while stack.last().map_or(false, |last| last.val < node.val) {
                stack.pop();
            }
            stack.push(node);
        }
        let mut head = None;
        while let Some(mut node) = stack.pop() {
            node.next = head;
            head = Some(node);
        }
        head
    }
}


fn main() {
    let head = to_list(vec![5,2,13,3,8]);
    let expected = to_list(vec![13,8]);
    print!("{}: ", "head".yellow());
    print_list(head.clone());
    print!("{}: ", "result".blue());
    let result = Solution::remove_nodes(head);
    print_list(result.clone());
    if result == expected {
        println!("{}", "Success!".green());
    } else {
        println!("{}", "Failed!".red());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_nodes() {
        // Test case 1: All elements are the same
        let head1 = to_list(vec![1, 1, 1, 1]);
        let expected1 = to_list(vec![1, 1, 1, 1]);
        let result1 = Solution::remove_nodes(head1);
        assert_eq!(result1, expected1);

        // Test case 2: Already in non-increasing order
        let head2 = to_list(vec![5, 4, 3, 2, 1]);
        let expected2 = to_list(vec![5, 4, 3, 2, 1]);
        let result2 = Solution::remove_nodes(head2);
        assert_eq!(result2, expected2);

        // Test case 3: Strictly increasing order
        let head3 = to_list(vec![1, 2, 8, 4, 5]);
        let expected3 = to_list(vec![8, 5]);
        let result3 = Solution::remove_nodes(head3);
        assert_eq!(result3, expected3);

        // Test case 4: Empty list
        let head4 = to_list(vec![]);
        let expected4 = to_list(vec![]);
        let result4 = Solution::remove_nodes(head4);
        assert_eq!(result4, expected4);

        // Test case 5: Single element
        let head5 = to_list(vec![100]);
        let expected5 = to_list(vec![100]);
        let result5 = Solution::remove_nodes(head5);
        assert_eq!(result5, expected5);
    }
}