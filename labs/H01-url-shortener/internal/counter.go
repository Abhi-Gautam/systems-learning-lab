package internal

import (
	"hash/fnv"
	"log"
	"sync"
	"sync/atomic"
)

// Verbose toggles per-call tracing inside Counter implementations.
// Off by default so benchmarks stay clean; flip on from the server
// when you want to WATCH the pipeline.
var Verbose bool

// Counter is the interface every ID-generation strategy implements.
// It returns the next monotonic uint64; the caller base62-encodes it.
type Counter interface {
	Next() uint64
	Name() string
}

// =====================================================================
// 1. NAIVE — one global counter behind one mutex.
//    This is what a junior writes. It WILL bottleneck under contention.
// =====================================================================

type NaiveCounter struct {
	mu  sync.Mutex
	cur uint64
}

func NewNaiveCounter(start uint64) *NaiveCounter {
	return &NaiveCounter{cur: start}
}

func (c *NaiveCounter) Next() uint64 {
	if Verbose {
		log.Printf("  [naive] acquiring the single global mutex...")
	}
	c.mu.Lock()
	if Verbose {
		log.Printf("  [naive] LOCKED. cur was %d, incrementing.", c.cur)
	}
	c.cur++
	v := c.cur
	c.mu.Unlock()
	if Verbose {
		log.Printf("  [naive] UNLOCKED. returning %d.", v)
	}
	return v
}

func (c *NaiveCounter) Name() string { return "naive" }

// =====================================================================
// 2. SHARDED — N independent counters; route by hashing some key.
//    Eliminates the single point of contention. Trade: IDs no longer
//    appear in a single monotonic sequence (still globally unique).
// =====================================================================

type ShardedCounter struct {
	shards []*shardSlot
	n      uint64
}

type shardSlot struct {
	mu  sync.Mutex
	cur uint64
	_   [40]byte // cache-line padding to avoid false sharing
}

func NewShardedCounter(nShards int, start uint64) *ShardedCounter {
	shards := make([]*shardSlot, nShards)
	// Pre-assign disjoint ranges: shard i owns IDs i, i+N, i+2N, ...
	// So each shard's "next" is unique modulo N. By construction no
	// two shards can mint the same ID.
	for i := 0; i < nShards; i++ {
		shards[i] = &shardSlot{cur: start + uint64(i)}
	}
	return &ShardedCounter{shards: shards, n: uint64(nShards)}
}

func (c *ShardedCounter) Next() uint64 {
	idx := pickShard(c.n)
	if Verbose {
		log.Printf("  [sharded] picked shard idx=%d of %d. acquiring shard's mutex...", idx, c.n)
	}
	s := c.shards[idx]
	s.mu.Lock()
	if Verbose {
		log.Printf("  [sharded] LOCKED shard %d. cur was %d, stepping by N=%d.", idx, s.cur, c.n)
	}
	s.cur += c.n // step by N so each shard stays in its lane
	v := s.cur
	s.mu.Unlock()
	if Verbose {
		log.Printf("  [sharded] UNLOCKED shard %d. returning %d (this is the %d-th ID in lane %d).", idx, v, (v-idx)/c.n, idx)
	}
	return v
}

func (c *ShardedCounter) Name() string { return "sharded" }

// =====================================================================
// 3. BLOCK — sharded + per-goroutine block pre-allocation.
//    Each worker reserves a block of K IDs in one locked op, then
//    serves K requests with zero coordination. Cuts contention by Kx.
//    This is what the H01 note recommends.
// =====================================================================

type BlockCounter struct {
	shards    []*shardSlot
	n         uint64
	blockSize uint64
	// per-goroutine state stored via sync.Pool to avoid global maps
	pool sync.Pool
}

type block struct {
	shard int
	next  uint64 // next ID to hand out
	end   uint64 // exclusive — when next >= end, refill
}

func NewBlockCounter(nShards int, start uint64, blockSize uint64) *BlockCounter {
	shards := make([]*shardSlot, nShards)
	for i := 0; i < nShards; i++ {
		shards[i] = &shardSlot{cur: start + uint64(i)}
	}
	c := &BlockCounter{shards: shards, n: uint64(nShards), blockSize: blockSize}
	c.pool.New = func() any { return &block{shard: -1} }
	return c
}

func (c *BlockCounter) Next() uint64 {
	b := c.pool.Get().(*block)
	defer c.pool.Put(b)
	if Verbose {
		log.Printf("  [block] got block from pool: next=%d end=%d shard=%d", b.next, b.end, b.shard)
	}
	if b.next >= b.end {
		// Refill: reserve blockSize IDs from one shard in a single lock op.
		idx := pickShard(c.n)
		if Verbose {
			log.Printf("  [block] block EMPTY (next>=end). REFILL: picked shard %d, locking...", idx)
		}
		s := c.shards[idx]
		s.mu.Lock()
		start := s.cur
		s.cur += c.blockSize * c.n
		s.mu.Unlock()
		b.shard = int(idx)
		b.next = start
		b.end = start + c.blockSize*c.n
		if Verbose {
			log.Printf("  [block] REFILLED from shard %d. reserved IDs [%d .. %d) — one lock for the next %d serves.", idx, b.next, b.end, c.blockSize)
		}
	} else if Verbose {
		log.Printf("  [block] FAST PATH: no lock taken. block has IDs [%d .. %d) remaining.", b.next, b.end)
	}
	v := b.next
	b.next += c.n
	if Verbose {
		log.Printf("  [block] returning %d. block now: next=%d end=%d. (put back to pool via defer)", v, b.next, b.end)
	}
	return v
}

func (c *BlockCounter) Name() string { return "block" }

// =====================================================================
// shard picker — cheap, lock-free, deterministic per goroutine but
// distributed across the shard set.
// =====================================================================

var pickSeed atomic.Uint64

func pickShard(n uint64) uint64 {
	v := pickSeed.Add(1)
	h := fnv.New64a()
	var buf [8]byte
	for i := 0; i < 8; i++ {
		buf[i] = byte(v >> (i * 8))
	}
	h.Write(buf[:])
	return h.Sum64() % n
}
