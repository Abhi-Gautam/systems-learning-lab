package ratelimiter

import "time"

type fixedWindow struct {
	limit       int
	window      time.Duration
	count       int
	windowStart time.Time
}

func newFixedWindow(
	limit int,
	window time.Duration,
	now time.Time,
) *fixedWindow {
	return &fixedWindow{
		limit:       limit,
		window:      window,
		windowStart: now,
	}
}

func (w *fixedWindow) allow(now time.Time) Decision {
	elapsed := now.Sub(w.windowStart)

	if elapsed >= w.window {
		w.count = 0
		w.windowStart = now
		elapsed = 0
	}

	if w.count >= w.limit {
		return Decision{
			Allowed:    false,
			RetryAfter: w.window - elapsed,
		}
	}

	w.count++

	return Decision{
		Allowed: true,
	}
}
