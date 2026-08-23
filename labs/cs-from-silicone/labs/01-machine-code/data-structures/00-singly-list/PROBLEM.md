# 00 — Singly Linked List (lab extra)

Not a LeetCode ID. Foundation for every pointer lab.

Implement an owning singly linked list of `int` using `Node*`.

API: `empty`, `size`, `push_front`, `push_back`, `pop_front`, `front`, `clear`,
`find`, `remove_first`. Store a count. `front` is only called when non-empty.
The destructor must delete every node. Disable copying. Target O(1) head operations,
O(n) search/removal, and O(n) clear. Do not use STL containers.

## Run

```bash
make test-00-singly-list
make test-00-singly-list ASAN=1
```
