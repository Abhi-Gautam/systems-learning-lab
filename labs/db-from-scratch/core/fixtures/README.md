# Fixtures

Reusable deterministic datasets for database internals learning.

## `user_records.tsv`

Format:

```text
set<TAB>id<TAB>name<TAB>note
```

Purpose:

- visualization demos in `src/main.rs`
- regression tests in `tests/`
- future scan/index benchmarks
- stable comparison across storage-engine milestones

Current sets:

- `tiny_basics`: tiny records for first sanity checks
- `b_tree_unsorted_keys`: deliberately scrambled IDs for seeing heap insertion order differ from B+Tree key order; named to print before `benchmark_seed` in demos
- `variable_lengths`: records with growing encoded byte sizes
- `unicode_utf8`: UTF-8 names to prove byte length != character count
- `deletion_candidates`: records meant for tombstone/delete tests
- `page_pressure`: wider records that force page-space behavior to become visible
- `benchmark_seed`: deterministic larger set for future scans/index benchmarks

Rule: keep this file deterministic. Add rows, but do not randomize row order unless a lab specifically introduces randomized workloads.
