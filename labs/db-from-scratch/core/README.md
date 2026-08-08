# db_core

Memory-first database internals core.

Current unit of work: one in-memory slotted page.

Start in `src/page.rs` with:

1. `Page::new`
2. header getters/setters
3. `free_space`
4. record encode/decode
5. `insert`
6. `get`
7. `delete`
8. `debug_print`
