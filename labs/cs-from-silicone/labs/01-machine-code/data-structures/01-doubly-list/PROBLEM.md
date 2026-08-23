# 01 — Doubly Linked List (lab extra)

Foundation for LRU. Implement a doubly linked list of `int`, preferably with one
circular dummy sentinel. Expose `Node*` so a caller can erase a known node in O(1).
Implement `empty`, `size`, `push_front`, `push_back`, `pop_front`, `pop_back`,
`front`, `back`, `insert_after(Node*, int)`, `erase(Node*)`, `find`, and `clear`.
Maintain `prev->next == node` and `next->prev == node` for every real node.
Destructor must free all real nodes and the sentinel.

## Run

```bash
make test-01-doubly-list
make test-01-doubly-list ASAN=1
```
