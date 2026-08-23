# 22 — LC 1206 Design Skiplist

LeetCode 1206. Implement `search`, `add`, `erase` in expected O(log n). Use
multi-level nodes with raw forward pointers and your own random-level choice.
No vectors. First make a deterministic fixed-level version, then add a simple
PRNG. Duplicates are allowed; erase one occurrence. Destructor frees every level
without double-freeing nodes.

## Run

```bash
make test-22-lc1206-skiplist
make test-22-lc1206-skiplist ASAN=1
```
