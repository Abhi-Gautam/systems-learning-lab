# Data Structures — from-scratch C++ (pointers only)

Build classic interview structures **without STL containers**. Every lab is a
small C++17 target: you own the nodes, links, destructors, and complexity.

Path: `labs/01-machine-code/data-structures/`

Prerequisite mindset: `02-memory-runtime/pointers/` (address = number, ownership).

## Rules

Read `RULES.md` first. Short version: `new`/`delete` + your `Node*` graphs.
No `vector` / `list` / `unordered_map` / `stack` / `queue` in implementations.

## Curriculum contract (do in order)

### Tier 0 — pointer plumbing

1. `00-singly-list` — Singly linked list (foundation, not LeetCode)
2. `01-doubly-list` — Doubly linked list + O(1) splice/unlink
3. `02-lc707-design-linked-list` — LC 707 Design Linked List

### Tier 1 — ends, rings, dual structures

4. `03-lc622-circular-queue` — LC 622 Design Circular Queue
5. `04-lc641-circular-deque` — LC 641 Design Circular Deque
6. `05-lc1670-front-middle-back-queue` — LC 1670 Design Front Middle Back Queue
7. `06-lc232-queue-using-stacks` — LC 232 Implement Queue using Stacks
8. `07-lc225-stack-using-queues` — LC 225 Implement Stack using Queues
9. `08-lc155-min-stack` — LC 155 Min Stack
10. `09-lc1381-stack-with-increment` — LC 1381 Design a Stack With Increment Operation

### Tier 2 — hashing + random

11. `10-lc705-hash-set` — LC 705 Design HashSet (chaining)
12. `11-lc706-hash-map` — LC 706 Design HashMap (chaining + resize)
13. `12-lc380-insert-delete-getrandom` — LC 380 Insert Delete GetRandom O(1)
14. `13-open-address-map` — Open-addressing hash map (lab extra, not LC)

### Tier 3 — recency / frequency caches

15. `14-lc146-lru-cache` — LC 146 LRU Cache (DIY DLL + DIY map or map of node*)
16. `15-lc460-lfu-cache` — LC 460 LFU Cache
17. `16-lc432-all-oone` — LC 432 All O`one Data Structure
18. `17-lc895-max-freq-stack` — LC 895 Maximum Frequency Stack

### Tier 4 — trees / tries / skiplist

19. `18-lc208-trie` — LC 208 Implement Trie
20. `19-lc211-word-dictionary` — LC 211 Design Add and Search Words
21. `20-bst` — Binary search tree insert/search/delete (lab extra)
22. `21-lc173-bst-iterator` — LC 173 Binary Search Tree Iterator
23. `22-lc1206-skiplist` — LC 1206 Design Skiplist

### Tier 5 — allocators / layout extras

24. `23-free-list-allocator` — Bump + free-list block allocator (lab extra)
25. `24-intrusive-list` — Intrusive doubly linked list (lab extra)
26. `25-binary-heap` — Binary heap over a raw array (lab extra)
27. `26-lc1472-browser-history` — LC 1472 Design Browser History

## Layout of each problem

```text
NN-name/
  PROBLEM.md   # full spec, constraints, examples, complexity target
  impl.h       # public API (do not break signatures tests rely on)
  impl.cpp     # YOU implement this (stubs only today)
  test.cpp     # harness — run to verify; do not weaken asserts
```

## How to build and test

From this directory:

```bash
cd /Volumes/mac-devlopment/systems-learning-lab/labs/cs-from-silicone/labs/01-machine-code/data-structures

# list targets
make help

# build + run tests for one problem (example)
make test-00-singly-list

# build only
make 00-singly-list

# run all that currently compile (stubs fail tests until you implement)
make test-all

# AddressSanitizer build of one target
make test-00-singly-list ASAN=1
```

Compiler default: `clang++ -std=c++17 -Wall -Wextra -Werror=return-type -O0 -g`

## What “done” means for a problem

1. `make test-<id>` exits 0
2. ASan clean (`ASAN=1`)
3. You can sketch node layout and point to which pointer moves on each op
4. You can state time/space for each public method

## Hermes contract for this lab

- You write all `impl.cpp` bodies.
- Chat gives specs, reviews your diffs, counterexamples, and silicon/layout notes.
- No solution dumps unless you explicitly ask for a patch on a stuck function.

## Observe points (every structure)

After each problem works, jot (in comments or a note):

- Where do nodes live (heap)? What does the head/tail/sentinel point at?
- How many pointer writes for insert/delete at head vs middle?
- Cache: pointer-chasing vs contiguous array — when did you feel it?
- Failure modes: empty structure, capacity 0/1, duplicate keys, destructor order
