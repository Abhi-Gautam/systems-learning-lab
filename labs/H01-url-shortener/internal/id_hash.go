package internal

import (
	"crypto/sha256"
	"encoding/binary"
	"sync/atomic"
)

// HashIDGen is the "let's just hash the URL" alternative.
// It exists in the lab so you can WATCH the birthday paradox bite.
//
// width = number of base62 chars to keep from the hash.
//   width=7  → 62^7 ≈ 3.5e12 slots — collisions essentially impossible
//   width=5  → 62^5 ≈ 9.2e8  slots — collisions rare but real
//   width=4  → 62^4 ≈ 1.5e7  slots — collisions become observable in seconds
//   width=3  → 62^3 ≈ 2.4e5  slots — collisions in dozens of writes (birthday paradox)
//
// The Counter interface returns a uint64; we encode the hash into a
// uint64 and mask it down to the requested bit-width's worth of base62
// digits so Base62() produces the right output length.
type HashIDGen struct {
	width      uint
	max        uint64 // 62^width
	collisions atomic.Uint64
	requests   atomic.Uint64
}

func NewHashIDGen(width uint) *HashIDGen {
	if width == 0 || width > 10 {
		panic("HashIDGen width must be 1..10")
	}
	max := uint64(1)
	for i := uint(0); i < width; i++ {
		max *= 62
	}
	return &HashIDGen{width: width, max: max}
}

// NextFor hashes the given URL into the configured slot space.
// It is NOT part of the Counter interface because hash IDs are a
// function of the input, not a monotonic counter.
func (h *HashIDGen) NextFor(url string) uint64 {
	h.requests.Add(1)
	sum := sha256.Sum256([]byte(url))
	v := binary.BigEndian.Uint64(sum[:8]) % h.max
	return v
}

// RecordCollision is called by the server when an INSERT failed
// because the ID was already taken. The bench reads these stats.
func (h *HashIDGen) RecordCollision() { h.collisions.Add(1) }

func (h *HashIDGen) Stats() (requests, collisions uint64) {
	return h.requests.Load(), h.collisions.Load()
}

func (h *HashIDGen) Name() string { return "hash" }
