# Rules — pointer-only C++ DS lab

You implement every structure with **raw nodes and pointers** (or an explicit
arena of nodes + indices acting as pointers). The point is ownership, linking,
and layout — not STL fluency.

## Allowed in `*.h` / student `*.cpp` (implementation)

- Language: C++17
- Headers: `<cstddef>`, `<cstdint>`, `<climits>`, `<cassert>`, `<new>`,
  `<utility>` (for `std::move` / `std::swap` only), `<initializer_list>` if useful
- Memory: `new` / `delete`, `new[]` / `delete[]`
- Your own `struct Node { ...; Node* next; ... };`
- Plain arrays you allocate (`new T[n]`) as **backing stores** (hash buckets,
  circular buffer storage, heap array)
- `nullptr`, references as function params

## Banned in implementation files

Do **not** include or use:

- `<vector>`, `<list>`, `<deque>`, `<stack>`, `<queue>`, `<forward_list>`
- `<map>`, `<set>`, `<unordered_map>`, `<unordered_set>`, `<multimap>`, ...
- `<array>` as a dynamic container substitute is fine only for **fixed** small
  tables (e.g. trie `Node* child[26]` can be a C array member — prefer that)
- `<algorithm>` containers helpers on library containers (write your own loops)
- `<memory>` smart pointers (`unique_ptr`, `shared_ptr`) — use raw + destructor
- Any Boost / Abseil / third-party container

Tests (`test.cpp`) may use `<cstdio>`, `<cassert>`, `<cstring>`, and the tiny
helpers in `common/test_util.h` only.

## Complexity contract

Unless a PROBLEM.md says otherwise:

- Document big-O in a short comment above each public method
- Prefer true O(1) where the LeetCode problem requires it (LRU get/put, etc.)
- No hidden O(n) scans that the API complexity forbids

## Memory contract

- Every `new` has a matching `delete` path (destructor and/or clear)
- No leaks under the provided tests (run with ASan — see README)
- Double-free and use-after-free count as fail

## How you work

1. Read `PROBLEM.md` in the problem folder
2. Fill `impl.cpp` (and only touch `impl.h` if you must add private helpers)
3. Do not change test expectations to make green — fix the structure
4. `make <problem-id>` then `make test-<problem-id>`
