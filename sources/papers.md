# Canonical Papers

The papers that power `/design-today` Deep dives. One row per paper. Cited by name + section number in HLD/LLD note "Where this shows up in production" and Mechanics walks.

These papers are **not duplicated** in the repo. They live wherever the user keeps them (typically ACM DL, arXiv, or the author's site). The composer relies on training-data knowledge to cite specifics; if uncertain about a number, the composer should WebFetch the paper.

---

## Distributed storage

### Dynamo (DeCandia et al. 2007)
**Powers**: H09 distributed cache · H17 DynamoDB · H23 leaderboard
**Key sections**: §4.4 vector clocks · §4.5 sloppy quorum + hinted handoff · §4.7 Merkle anti-entropy
**One-line**: AP leaderless KV store; introduces the three-mechanism stack (sloppy quorum + hinted handoff + Merkle anti-entropy) for write-availability under partition.

### Spanner (Corbett et al. 2012)
**Powers**: H39 Spanner · H14 Stripe payment (cross-region transactions)
**Key sections**: §3 TrueTime · §4 Paxos groups · §5 transactions
**One-line**: Globally-distributed SQL with external consistency via bounded clock uncertainty (TrueTime); 2PC layered on Paxos-replicated shards.

### Bigtable (Chang et al. 2006)
**Powers**: H17 DynamoDB · H31 TSDB · H24 log analytics
**Key sections**: §5.3 SSTable · §6 refinement (compaction)
**One-line**: Wide-column distributed table; SSTable + memtable + WAL is the foundational LSM-tree design.

### GFS (Ghemawat et al. 2003)
**Powers**: H07 Dropbox · H18 GFS · object-storage discussions
**Key sections**: §3 architecture · §4 master operation
**One-line**: Single-master + chunkservers; append-only semantics; replication factor 3.

### Chubby (Burrows 2006)
**Powers**: H39 Spanner (lock service) · H17 (coordination)
**One-line**: Distributed coarse-grained lock service backed by Paxos; "the GFS lock manager."

### ZooKeeper (Hunt et al. 2010)
**Powers**: H16 Kafka (controller election) · H17 (membership)
**One-line**: Coordination service with linearizable writes and FIFO client ordering; ZAB protocol.

---

## Consensus

### Paxos Made Simple (Lamport 2001)
**Powers**: H39 Spanner · H17 DynamoDB (variant) · H16 Kafka (ZK)
**One-line**: The original consensus algorithm. Hard to implement correctly; almost always paired with Multi-Paxos in practice.

### Raft (Ongaro & Ousterhout 2014)
**Powers**: H17 DynamoDB · H39 Spanner alternative · H27 RAG (index replication)
**Key sections**: §5 basic Raft · §6 cluster membership
**One-line**: Consensus designed for understandability. Leader election + log replication + safety in three modules.

---

## Streaming + batch

### Kafka (Kreps et al. 2011)
**Powers**: H16 Kafka · H20 ad-click aggregator · H44 model routing
**Key sections**: §3 architecture · §4 design choices
**One-line**: Append-only partitioned log as a primitive; consumers track their own offsets; replication via ISR.

### MapReduce (Dean & Ghemawat 2004)
**Powers**: H20 ad-click aggregator (batch leg) · H42 distributed training (loosely)
**One-line**: Distributed batch processing primitive; map → shuffle → reduce; fault tolerance via re-execution.

### MillWheel (Akidau et al. 2013)
**Powers**: H20 ad-click aggregator · H32 distributed tracing
**Key sections**: §3 watermarks · §4 exactly-once
**One-line**: Streaming with watermarks for handling late data; exactly-once semantics via dedup + atomicity.

---

## Time-series + observability

### Gorilla (Pelkonen et al. 2015)
**Powers**: H31 TSDB · H20 streaming aggregation
**Key sections**: §4 delta-of-delta · §5 XOR compression
**One-line**: Facebook's in-memory TSDB; delta-delta + XOR compression gives ~12 bytes/sample (down from 16+).

### Dapper (Sigelman et al. 2010)
**Powers**: H32 distributed tracing
**Key sections**: §2 trace tree · §3 instrumentation
**One-line**: Google's tracing system; introduces trace + span data model and head sampling.

---

## ML serving + retrieval

### vLLM / PagedAttention (Kwon et al. 2023)
**Powers**: H26 LLM serving · H29 agent orchestration
**Key sections**: §3 PagedAttention · §4 scheduler
**One-line**: KV cache memory management via OS-style paging; enables 2-24x throughput vs naive serving.

### Orca (Yu et al. 2022)
**Powers**: H26 LLM serving
**One-line**: Continuous batching — iteration-level scheduling that lets new requests join a running batch.

### HNSW (Malkov & Yashunin 2018)
**Powers**: H27 RAG · H45 vector DB
**One-line**: Hierarchical Navigable Small World graphs for approximate nearest neighbor; logarithmic search complexity at high recall.

---

## Classical IR

### BM25 (Robertson & Walker 1994)
**Powers**: H27 RAG (hybrid search) · H11 web crawler · H12 autocomplete
**One-line**: Probabilistic ranking function for keyword retrieval; still the strongest non-neural baseline.

---

## Cluster + infrastructure

### Borg (Verma et al. 2015)
**Powers**: H42 distributed training · scheduling discussions
**One-line**: Google's cluster manager; cell scheduler + per-machine borglet. Predecessor to Kubernetes.

### Memcached@FB (Nishtala et al. 2013)
**Powers**: H09 distributed cache · H08 Instagram feed
**Key sections**: §3 leases · §4 regional pools
**One-line**: How Facebook scaled Memcached to billions of QPS; lease tokens for stampede protection; cold cluster warmup.

---

## LLM serving + inference infra

### Orca (Yu et al. 2022, OSDI)
**Powers**: L47 continuous-batching scheduler · L50 KV-cache block manager
**Key sections**: §3 iteration-level scheduling · selective batching
**One-line**: Introduced continuous (iteration-level) batching — admit/retire requests every decode step instead of per static batch; the foundation of modern LLM serving throughput.

### PagedAttention / vLLM (Kwon et al. 2023, SOSP)
**Powers**: L50 KV-cache block manager · L47 continuous-batching scheduler
**Key sections**: §4 block table & logical→physical mapping · copy-on-write sharing
**One-line**: Treats the KV cache like OS paging — fixed-size blocks, near-zero fragmentation, prefix sharing via refcounted blocks; ~2-4× throughput over contiguous allocation.

### BPE for subwords (Sennrich et al. 2016)
**Powers**: L46 BPE tokenizer
**Key sections**: §3.2 the merge algorithm
**One-line**: Byte-pair encoding for open-vocabulary tokenization; merge most-frequent adjacent pairs in rank order. Byte-level variants add lossless fallback for any input.

### ReAct (Yao et al. 2023)
**Powers**: L48 agent orchestration loop
**Key sections**: interleaved reason→act trace · termination
**One-line**: Interleaves model reasoning with tool actions in a single loop; the conceptual core of the observe→think→act agent runtime.

---

## Probabilistic / streaming structures (observability)

### DDSketch (Masson, Rim, Lee 2019, VLDB)
**Powers**: L52 mergeable quantile sketch
**Key sections**: §3 logarithmic mapping (index = ⌈log_γ(x)⌉, γ = (1+α)/(1−α)) · §4 mergeability & bucket collapsing
**One-line**: Datadog's quantile sketch — log-spaced buckets give a *relative*-error guarantee (not absolute), and a fixed γ makes two sketches mergeable by summing bucket counts, enabling distributed percentile computation.

### Count-Min Sketch (Cormode & Muthukrishnan 2005)
**Powers**: L55 streaming heavy-hitters / top-K
**Key sections**: §3 the d×w counter matrix · the (ε, δ) bound (w = ⌈e/ε⌉, d = ⌈ln 1/δ⌉)
**One-line**: Fixed-memory frequency estimator; d hash rows, estimate = MIN across rows so collisions (which only add) cancel. Pair with a min-heap to enumerate the top-K it can't list on its own.

---

## Adding a paper

1. Add a section under the appropriate category.
2. Include `**Powers**:` (which problems cite it) and `**Key sections**:` (specific paper references the composer might quote).
3. If the paper introduces a named concept the composer should know (e.g., "watermarks"), tag it in `**One-line**:`.
