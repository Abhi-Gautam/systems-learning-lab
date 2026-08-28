package ratelimiter

import "time"

// algorithm represents one per-user rate-limiting algorithm.
//
// The coordinator owns user lookup and synchronization. Implementations own
// their algorithm-specific state and decision calculation.

type Decision struct {
	Allowed    bool
	RetryAfter time.Duration
}
