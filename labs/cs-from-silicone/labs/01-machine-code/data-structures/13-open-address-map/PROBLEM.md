# 13 — Open Addressing Hash Map (lab extra)

Implement an integer-to-integer hash map with open addressing and linear probing.
Use a raw slot array plus explicit slot state (empty/occupied/tombstone), not
chaining. Implement put/get/remove, collision probing, tombstone reuse, and
resize. Explain why a tombstone cannot be treated as empty. Target expected O(1).

## Run

```bash
make test-13-open-address-map
make test-13-open-address-map ASAN=1
```
