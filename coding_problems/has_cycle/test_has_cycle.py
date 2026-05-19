from coding_problems.py_coding_lib.listnode import ListNode
from .has_cycle import Solution

def build_list(values, cycle_pos=-1):
    if not values:
        return None

    nodes = [ListNode(value) for value in values]
    for current, next_node in zip(nodes, nodes[1:]):
        current.next = next_node

    if cycle_pos != -1:
        nodes[-1].next = nodes[cycle_pos]

    return nodes[0]


def test_has_cycle_returns_true_for_cycled_list():
    head = build_list([3, 2, 0, -4], cycle_pos=1)

    assert Solution().hasCycle(head) is True


def test_has_cycle_returns_false_for_single_node():
    head = build_list([1])

    assert Solution().hasCycle(head) is False


def test_has_cycle_returns_false_for_empty_list():
    assert Solution().hasCycle(None) is False


def test_has_cycle_returns_false_for_non_cycled_list():
    head = build_list([1, 2, 3, 4])

    assert Solution().hasCycle(head) is False
