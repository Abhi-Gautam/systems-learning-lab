# 10 — LC 705 Design HashSet

LeetCode 705. Implement `add(key)`, `remove(key)`, `contains(key)` using an
array of bucket head pointers and separate-chaining nodes. No unordered_set.
Choose a bucket count, explain hash and collision behavior, and optionally resize.
Target expected O(1), worst-case O(n). Destructor deletes every chain.

## Run

```bash
make test-10-lc705-hash-set
make test-10-lc705-hash-set ASAN=1
```
