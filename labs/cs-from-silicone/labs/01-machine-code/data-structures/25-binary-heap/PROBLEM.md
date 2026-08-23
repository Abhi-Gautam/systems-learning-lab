# 25 — Binary Heap over Raw Array (lab extra)

Implement a min-heap of integers over `new int[capacity]`. API: `push`, `pop`,
`peek`, `empty`, `size`. Grow the raw array when full. Write parent/child index
arithmetic and sift-up/sift-down yourself. No priority_queue/vector. Then explain
why the array layout is more cache-friendly than pointer-linked heap nodes.

## Run

```bash
make test-25-binary-heap
make test-25-binary-heap ASAN=1
```
