# 09 — LC 1381 Design a Stack With Increment Operation

LeetCode 1381. Fixed-capacity stack with `push`, `pop`, and `increment(k,val)`.
`increment` adds val to the bottom k elements. Use your own linked stack nodes
and achieve O(1) `increment` via lazy propagation metadata, or implement a first
O(k) version and record why it is not optimal. No vector/stack.

## Run

```bash
make test-09-lc1381-stack-with-increment
make test-09-lc1381-stack-with-increment ASAN=1
```
