use std::cell::RefCell;
use std::rc::Rc;

type Link = Option<Rc<RefCell<ListNode>>>;

#[derive(Debug)]
struct ListNode {
    _val: i32,
    next: Link,
}

impl ListNode {
    fn new(val: i32) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self { _val: val, next: None }))
    }
}

struct Solutions;

impl Solutions {
    pub fn has_cycle(head: Link) -> bool {
        let mut slow = head.clone();
        let mut fast = head;

        while let Some(fast_node) = fast {
            slow = slow.and_then(|node| node.borrow().next.clone());
            fast = fast_node
                .borrow()
                .next
                .clone()
                .and_then(|node| node.borrow().next.clone());

            if let (Some(slow_node), Some(fast_node)) = (&slow, &fast) {
                if Rc::ptr_eq(slow_node, fast_node) {
                    return true;
                }
            }
        }

        false
    }
}

fn list_with_cycle(values: Vec<i32>, pos: Option<usize>) -> Link {
    if values.is_empty() {
        return None;
    }

    let nodes: Vec<_> = values.into_iter().map(ListNode::new).collect();

    for i in 0..nodes.len() - 1 {
        nodes[i].borrow_mut().next = Some(nodes[i + 1].clone());
    }

    if let Some(pos) = pos {
        nodes[nodes.len() - 1].borrow_mut().next = Some(nodes[pos].clone());
    }

    Some(nodes[0].clone())
}

fn main() {
    let cycled_head = list_with_cycle(vec![3, 2, 0, -4], Some(1));
    println!("Cycled list has cycle: {}", Solutions::has_cycle(cycled_head));

    let non_cycled_head = list_with_cycle(vec![1, 2, 3, 4], None);
    println!(
        "Non-cycled list has cycle: {}",
        Solutions::has_cycle(non_cycled_head)
    );
}
