# 15 — LC 460 LFU Cache

LeetCode 460. Implement O(1) average `get`/`put`. Compose: key -> node index,
frequency -> doubly linked list of nodes ordered by recency, and `min_freq`.
On access increment frequency and move node. On capacity overflow evict the LRU
node in the minimum-frequency list. No STL containers. This is a capstone in
multiple linked structures and ownership.

## Run

```bash
make test-15-lc460-lfu-cache
make test-15-lc460-lfu-cache ASAN=1
```
