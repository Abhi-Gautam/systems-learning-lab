# 04 — LC 641 Design Circular Deque

LeetCode 641: https://leetcode.com/problems/design-circular-deque/

Implement a fixed-capacity circular deque with `insertFront`, `insertLast`,
`deleteFront`, `deleteLast`, `getFront`, `getRear`, `isEmpty`, `isFull`.
Use a raw `int*` ring buffer and explicit index arithmetic. Every operation O(1).
Test wrap-around in both directions and capacities 0, 1, and 3.

## Run

```bash
make test-04-lc641-circular-deque
make test-04-lc641-circular-deque ASAN=1
```
