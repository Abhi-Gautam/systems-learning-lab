package internal

import (
	"container/list"
	"sync"
	"sync/atomic"
)

// LRUCache — small, thread-safe, with hit/miss counters so the
// read-path experiments can report hit rate directly.
type LRUCache struct {
	mu       sync.Mutex
	cap      int
	ll       *list.List
	items    map[string]*list.Element
	hits     atomic.Uint64
	misses   atomic.Uint64
}

type lruEntry struct{ k, v string }

func NewLRUCache(cap int) *LRUCache {
	return &LRUCache{cap: cap, ll: list.New(), items: make(map[string]*list.Element, cap)}
}

func (c *LRUCache) Get(k string) (string, bool) {
	c.mu.Lock()
	defer c.mu.Unlock()
	if e, ok := c.items[k]; ok {
		c.ll.MoveToFront(e)
		c.hits.Add(1)
		return e.Value.(lruEntry).v, true
	}
	c.misses.Add(1)
	return "", false
}

func (c *LRUCache) Put(k, v string) {
	c.mu.Lock()
	defer c.mu.Unlock()
	if e, ok := c.items[k]; ok {
		e.Value = lruEntry{k, v}
		c.ll.MoveToFront(e)
		return
	}
	if c.ll.Len() >= c.cap {
		oldest := c.ll.Back()
		if oldest != nil {
			c.ll.Remove(oldest)
			delete(c.items, oldest.Value.(lruEntry).k)
		}
	}
	c.items[k] = c.ll.PushFront(lruEntry{k, v})
}

func (c *LRUCache) Stats() (hits, misses uint64) {
	return c.hits.Load(), c.misses.Load()
}
