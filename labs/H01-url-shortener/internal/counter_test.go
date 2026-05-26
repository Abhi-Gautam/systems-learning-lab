package internal

import (
	"runtime"
	"sync"
	"sync/atomic"
	"testing"
	"time"
)

// These microbenchmarks strip away HTTP/JSON/store and pound the
// Counter implementations directly. This is where the contention
// curve becomes visible — at the HTTP layer the network + JSON
// dominate and hide the mutex cost.
//
// Run with:
//   go test -bench=. -benchtime=2s ./internal
//
// To pin to N cores:
//   go test -bench=. -cpu=1,4,16,64 -benchtime=2s ./internal

// =====================================================================
// Single-threaded baseline — best case for each counter.
// =====================================================================

func BenchmarkCounterNaive_Serial(b *testing.B) {
	c := NewNaiveCounter(1)
	for i := 0; i < b.N; i++ {
		_ = c.Next()
	}
}

func BenchmarkCounterSharded_Serial(b *testing.B) {
	c := NewShardedCounter(4, 1)
	for i := 0; i < b.N; i++ {
		_ = c.Next()
	}
}

func BenchmarkCounterBlock_Serial(b *testing.B) {
	c := NewBlockCounter(4, 1, 1000)
	for i := 0; i < b.N; i++ {
		_ = c.Next()
	}
}

// =====================================================================
// Contended — multiple goroutines hammering the SAME counter.
// This is where naive falls off a cliff.
// =====================================================================

func BenchmarkCounterNaive_Parallel(b *testing.B) {
	c := NewNaiveCounter(1)
	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			_ = c.Next()
		}
	})
}

func BenchmarkCounterSharded_Parallel(b *testing.B) {
	c := NewShardedCounter(4, 1)
	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			_ = c.Next()
		}
	})
}

func BenchmarkCounterBlock_Parallel(b *testing.B) {
	c := NewBlockCounter(4, 1, 1000)
	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			_ = c.Next()
		}
	})
}

// =====================================================================
// Throughput probe — measure ops/sec for each, with N concurrent
// goroutines, for a fixed wall-clock window. Easier to reason about
// than ns/op for the contention story.
// =====================================================================

func TestThroughputComparison(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping in -short")
	}
	for _, workers := range []int{1, 4, 16, 64, 256} {
		t.Logf("--- workers=%d (GOMAXPROCS=%d) ---", workers, runtime.GOMAXPROCS(0))

		runFor := func(name string, next func() uint64) {
			var ops uint64
			var wg sync.WaitGroup
			stop := make(chan struct{})
			for i := 0; i < workers; i++ {
				wg.Add(1)
				go func() {
					defer wg.Done()
					var local uint64
					for {
						select {
						case <-stop:
							atomic.AddUint64(&ops, local)
							return
						default:
							_ = next()
							local++
						}
					}
				}()
			}
			time.Sleep(500 * time.Millisecond)
			close(stop)
			wg.Wait()
			t.Logf("  %-8s %12d ops/sec", name, ops*2) // *2 since we ran 500ms
		}

		naive := NewNaiveCounter(1)
		sharded := NewShardedCounter(4, 1)
		block := NewBlockCounter(4, 1, 1000)

		runFor("naive", naive.Next)
		runFor("sharded", sharded.Next)
		runFor("block", block.Next)
	}
}
