# 12 — LC 380 Insert Delete GetRandom O(1)

LeetCode 380. Implement `insert(val)`, `remove(val)`, and `getRandom()`.
Expected O(1) requires two homemade structures: a contiguous raw array of values
and a hash table mapping value to its array index. Removal uses swap-with-last,
then repairs the moved value's index. No vector, unordered_map, or RNG library;
for deterministic tests expose a private/simple deterministic choice first, then
add randomness as a separate milestone.

## Run

```bash
make test-12-lc380-insert-delete-getrandom
make test-12-lc380-insert-delete-getrandom ASAN=1
```
