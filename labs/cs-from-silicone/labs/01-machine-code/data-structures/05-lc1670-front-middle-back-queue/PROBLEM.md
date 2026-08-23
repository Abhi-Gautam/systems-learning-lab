# 05 — LC 1670 Front Middle Back Queue

LeetCode 1670: https://leetcode.com/problems/design-front-middle-back-queue/

Implement `pushFront`, `pushMiddle`, `pushBack`, `popFront`, `popMiddle`,
`popBack`; empty pops return -1. Use a DIY doubly linked list. The interesting
constraint is maintaining `mid_` as size changes. Follow LeetCode's middle rule:
for even size, `popMiddle` removes the left/first middle; `pushMiddle` inserts
before the current right/second middle. All operations should be O(1) after the
first correct implementation.

## Run

```bash
make test-05-lc1670-front-middle-back-queue
make test-05-lc1670-front-middle-back-queue ASAN=1
```
