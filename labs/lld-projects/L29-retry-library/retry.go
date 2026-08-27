package retry

import (
	"context"
	"errors"
	"math/rand"
	"time"
)

type ErrorClass uint8

const (
	Terminal ErrorClass = iota
	Retryable
)

type Config struct {
	Attempts int
	Backoff  Backoff
}

type ClassifiedError struct {
	Class ErrorClass
	Err   error
}

func (e ClassifiedError) Unwrap() error {
	return e.Err
}

func (e ClassifiedError) Error() string {
	return e.Err.Error()
}

func RetryableError(err error) error {
	return ClassifiedError{
		Class: Retryable,
		Err:   err,
	}
}

func TerminalError(err error) error {
	return ClassifiedError{
		Class: Terminal,
		Err:   err,
	}
}

func isRetryable(err error) bool {
	var classified ClassifiedError

	if errors.As(err, &classified) {
		return classified.Class == Retryable
	}

	return false
}

type Backoff interface {
	Next(attempt int) time.Duration
}

type FixedBackoff struct {
	Delay time.Duration
}

func (f FixedBackoff) Next(attempt int) time.Duration {
	return f.Delay
}

type ExponentialBackoff struct {
	Base time.Duration
	Max  time.Duration
}

func (e ExponentialBackoff) Next(attempt int) time.Duration {
	delay := e.Base

	for i := 1; i < attempt; i++ {
		delay *= 2
		if delay > e.Max {
			delay = e.Max
		}
	}
	return delay
}

type JitteredBackoff struct {
	Inner Backoff
}

func (j JitteredBackoff) Next(attempt int) time.Duration {
	delay := j.Inner.Next(attempt)

	if delay <= 1 {
		return delay
	}

	half := delay / 2
	jitter := rand.Int63n(int64(half))

	return half + time.Duration(jitter)
}

func Do[T any](ctx context.Context, cfg Config, operation func() (T, error)) (T, error) {
	var zero T
	for attempt := 1; attempt <= cfg.Attempts; attempt++ {
		result, err := operation()
		if err == nil {
			return result, nil
		}
		if !isRetryable(err) || attempt == cfg.Attempts {
			return zero, err
		}
		if cfg.Backoff != nil {
			delay := cfg.Backoff.Next(attempt)
			waitErr := wait(ctx, delay)
			if waitErr != nil {
				return zero, waitErr
			}
		}
	}
	return zero, nil
}

func wait(ctx context.Context, delay time.Duration) error {
	timer := time.NewTimer(delay)
	defer timer.Stop()

	select {
	case <-timer.C:
		return nil
	case <-ctx.Done():
		return ctx.Err()
	}
}
