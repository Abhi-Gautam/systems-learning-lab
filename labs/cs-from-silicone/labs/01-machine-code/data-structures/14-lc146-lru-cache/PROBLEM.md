# 14 — LC 146 LRU Cache

LeetCode 146. Implement `get(key)` and `put(key,value)` with O(1) average time.
Use a DIY doubly linked list ordered MRU -> LRU and a homemade hash index from key
to `Node*`. On get/overwrite move node to front; on capacity overflow evict tail.
Do not use list, map, unordered_map, or any standard container. Destructor must
free list nodes and the index. This is the flagship pointer-composition lab.

## Run

```bash
make test-14-lc146-lru-cache
make test-14-lc146-lru-cache ASAN=1
```
