package ratelimiter

import (
	"sync"
	"time"
)

type Limiter interface {
	Allow(userID string) Decision
}

type userState interface {
	allow(now time.Time) Decision
}

type userStateFactory func(time.Time) userState

type rateLimiter struct {
	mu           sync.Mutex
	states       map[string]userState
	newUserState userStateFactory
	now          func() time.Time
}

func newRateLimiter(newUserState userStateFactory, now func() time.Time) *rateLimiter {
	return &rateLimiter{
		newUserState: newUserState,
		now:          now,
		states:       make(map[string]userState),
	}
}

func (r *rateLimiter) Allow(userID string) Decision {
	r.mu.Lock()
	defer r.mu.Unlock()
	now := r.now()
	state, ok := r.states[userID]
	if !ok {
		state = r.newUserState(now)
		r.states[userID] = state
	}
	return state.allow(now)
}
