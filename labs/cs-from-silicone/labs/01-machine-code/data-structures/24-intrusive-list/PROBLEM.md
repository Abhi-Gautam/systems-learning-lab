# 24 — Intrusive Doubly Linked List (lab extra)

Implement an intrusive list: the payload object owns its `prev`/`next` link
fields, while the list owns no payload memory. API: `push_front(Node*)`,
`push_back(Node*)`, `remove(Node*)`, `front`, `back`, `empty`, `clear_links`.
Never delete payloads. Test moving the same object between positions and explain
why intrusive lists avoid per-node allocation.

## Run

```bash
make test-24-intrusive-list
make test-24-intrusive-list ASAN=1
```
