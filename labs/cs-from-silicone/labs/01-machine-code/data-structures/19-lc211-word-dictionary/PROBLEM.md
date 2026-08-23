# 19 — LC 211 Design Add and Search Words

LeetCode 211. Implement `addWord(word)` and `search(word)`, where `.` matches
any one character. Use your own trie nodes with 26 child pointers. Search with
explicit recursion/backtracking; no regex or containers. Destructor recursively
frees the graph. Document recursion depth and branching cost.

## Run

```bash
make test-19-lc211-word-dictionary
make test-19-lc211-word-dictionary ASAN=1
```
