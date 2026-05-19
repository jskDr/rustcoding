from typing import Optional
from coding_problems.py_coding_lib.listnode import ListNode


class Solution:
    def hasCycle(self, head: Optional[ListNode]) -> bool:
        slow = head
        fast = head
        
        while slow and fast:
            slow = slow.next
            if fast.next:
                fast = fast.next.next
            else:
                return False
            if slow == fast:
                return True
        return False