# 23 — Free-list Allocator (lab extra)

Implement a small allocator over one raw byte buffer obtained with `new unsigned
char[capacity]`. Start with bump allocation, then add a free list and coalescing.
API: `allocate(bytes)`, `deallocate(ptr)`, `bytes_free()`. Store block headers
inside the buffer using pointers/offsets. Define alignment, splitting, invalid
free behavior, and fragmentation tests. Do not call malloc/free/new per request.

## Run

```bash
make test-23-free-list-allocator
make test-23-free-list-allocator ASAN=1
```
