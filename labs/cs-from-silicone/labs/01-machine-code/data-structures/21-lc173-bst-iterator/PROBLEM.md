# 21 — LC 173 Binary Search Tree Iterator

LeetCode 173. Given a binary tree root, implement `next()` and `hasNext()` for
in-order traversal. Use your own linked stack of `TreeNode*`; no std::stack and
no recursive traversal state. In this lab define `TreeNode` in the header and
add a destructor helper only for the test-owned tree if needed. Target amortized
O(1) next and O(h) auxiliary memory.

## Run

```bash
make test-21-lc173-bst-iterator
make test-21-lc173-bst-iterator ASAN=1
```
