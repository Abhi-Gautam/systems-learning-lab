# 02 — LC 707 Design Linked List

LeetCode 707: https://leetcode.com/problems/design-linked-list/

Implement `MyLinkedList`: constructor; `get(index)` returning -1 when invalid;
`addAtHead`; `addAtTail`; `addAtIndex(index,val)` inserting before index, appending
when index == size and doing nothing when index > size; `deleteAtIndex(index)`.
Use your own nodes. Keep size and optionally tail. Target O(1) head/tail and O(n)
indexed operations. At most 2000 calls in the original problem.

## Run

```bash
make test-02-lc707-design-linked-list
make test-02-lc707-design-linked-list ASAN=1
```
