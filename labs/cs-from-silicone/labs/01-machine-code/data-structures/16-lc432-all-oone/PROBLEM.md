# 16 — LC 432 All O(1) Data Structure

LeetCode 432. Implement `inc(key)`, `dec(key)`, `getMaxKey()`, `getMinKey()`.
All operations O(1). Use a doubly linked list of frequency buckets, each bucket
holding your own linked set/list of string keys. Because this lab bans std::string,
use fixed C strings or design a small owned string type first and document it.
No map/set/unordered containers. Empty max/min returns an empty C string.

## Run

```bash
make test-16-lc432-all-oone
make test-16-lc432-all-oone ASAN=1
```
