package internal

import (
	"context"
	"errors"
	"sync"
	"time"

	"github.com/redis/go-redis/v9"
)

var ErrCollision = errors.New("id collision: a different URL already maps to this id")
var ErrNotFound = errors.New("not found")

// Store is short_id → long_url with collision detection on write.
type Store interface {
	// PutIfAbsent atomically stores id→url IFF id is unused.
	// Returns ErrCollision if id is already taken by a different url.
	// Returns nil if newly inserted OR if id→url already matches (idempotent).
	PutIfAbsent(ctx context.Context, id, url string) error
	Get(ctx context.Context, id string) (string, error)
}

// =====================================================================
// In-memory store — for fast iteration and the counter experiments.
// Sharded mutex map to avoid global lock contention swamping the
// counter benchmarks we actually care about.
// =====================================================================

type MemStore struct {
	buckets [256]*memBucket
}

type memBucket struct {
	mu sync.RWMutex
	m  map[string]string
}

func NewMemStore() *MemStore {
	s := &MemStore{}
	for i := range s.buckets {
		s.buckets[i] = &memBucket{m: make(map[string]string)}
	}
	return s
}

func (s *MemStore) bucket(id string) *memBucket {
	// FNV-ish: just sum the bytes.
	var h byte
	for i := 0; i < len(id); i++ {
		h ^= id[i]
	}
	return s.buckets[h]
}

func (s *MemStore) PutIfAbsent(_ context.Context, id, url string) error {
	b := s.bucket(id)
	b.mu.Lock()
	defer b.mu.Unlock()
	if existing, ok := b.m[id]; ok {
		if existing == url {
			return nil // idempotent
		}
		return ErrCollision
	}
	b.m[id] = url
	return nil
}

func (s *MemStore) Get(_ context.Context, id string) (string, error) {
	b := s.bucket(id)
	b.mu.RLock()
	defer b.mu.RUnlock()
	if v, ok := b.m[id]; ok {
		return v, nil
	}
	return "", ErrNotFound
}

// =====================================================================
// Redis-backed store — for the read-path experiments and to feel the
// network RTT cost vs in-process.
// =====================================================================

type RedisStore struct {
	client *redis.Client
}

func NewRedisStore(addr string) *RedisStore {
	c := redis.NewClient(&redis.Options{
		Addr:         addr,
		PoolSize:     200,
		ReadTimeout:  500 * time.Millisecond,
		WriteTimeout: 500 * time.Millisecond,
	})
	return &RedisStore{client: c}
}

func (s *RedisStore) PutIfAbsent(ctx context.Context, id, url string) error {
	// SETNX returns 1 if new, 0 if key existed.
	ok, err := s.client.SetNX(ctx, "u:"+id, url, 0).Result()
	if err != nil {
		return err
	}
	if !ok {
		existing, err := s.client.Get(ctx, "u:"+id).Result()
		if err != nil {
			return err
		}
		if existing == url {
			return nil
		}
		return ErrCollision
	}
	return nil
}

func (s *RedisStore) Get(ctx context.Context, id string) (string, error) {
	v, err := s.client.Get(ctx, "u:"+id).Result()
	if errors.Is(err, redis.Nil) {
		return "", ErrNotFound
	}
	return v, err
}
