# 03 — LC 622 Design Circular Queue

LeetCode 622: https://leetcode.com/problems/design-circular-queue/

Implement a fixed-capacity circular queue: `MyCircularQueue(k)`, `enQueue`,
`deQueue`, `Front`, `Rear`, `isEmpty`, `isFull`. Use `new int[k]`, a front index,
and count (or equivalent). No vector/deque/queue. All operations must be O(1).
Handle capacity 0 defensively even if the original constraints normally avoid it.

## Run

```bash
make test-03-lc622-circular-queue
make test-03-lc622-circular-queue ASAN=1
```
