# 20 — Binary Search Tree (lab extra)

Implement a binary search tree of unique integers: `insert`, `contains`,
`erase`, `min`, `max`, and destructor. Handle leaf, one-child, and two-child
delete cases. Do not use recursion for everything automatically: implement one
operation iteratively and compare pointer movement. Optional traversal callback.
Target average O(log n), worst O(n), with explicit ownership of every `TreeNode*`.

## Run

```bash
make test-20-bst
make test-20-bst ASAN=1
```
