package elevator

import (
	"context"
	"sync"
)

type Direction int

const (
	DirIdle Direction = iota
	DirUp
	DirDown
)

// Request models a hall call (FromFloor + Dir) or a car call (ToFloor set).
type Request struct {
	FromFloor int
	ToFloor   int
	Dir       Direction
}

// State is the car's behavioral state; transitions are enforced here.
type State interface {
	OnRequest(*Elevator, Request)
	OnArrival(*Elevator, int)
}

// Elevator is one car. `stops` is a sorted set so the next stop in the
// current direction is an O(log n) lookup, not an O(n) scan.
type Elevator struct {
	mu        sync.Mutex
	id        int
	floor     int
	direction Direction
	stops     *sortedSet // ordered ints; see Deep dive 1
	state     State
}

func (e *Elevator) enqueue(r Request) { /* push to per-car channel */ }

// sortedSet is intentionally unexported — the concrete impl is part of Deep dive 1.
type sortedSet struct{}

// Scheduler decides which car serves a request. Strategy seam.
type Scheduler interface {
	Assign(req Request, cars []*Elevator) *Elevator
}

// NearestCarScheduler scores each car by directional distance.
type NearestCarScheduler struct{ idlePenalty, sameDirBonus int }

func (s *NearestCarScheduler) Assign(req Request, cars []*Elevator) *Elevator {
	var best *Elevator
	bestScore := 1 << 30
	for _, c := range cars {
		if sc := s.score(req, c); sc < bestScore {
			best, bestScore = c, sc
		}
	}
	return best // never nil: N>=1 cars always exist
}

func (s *NearestCarScheduler) score(req Request, c *Elevator) int { /* Deep dive 1 */ return 0 }

// ElevatorBank is the aggregate root.
type ElevatorBank struct {
	elevators []*Elevator
	scheduler Scheduler
	inbox     chan Request
}

func NewElevatorBank(n int, sched Scheduler) *ElevatorBank { /* ... */ return nil }

// Submit enqueues a request and returns immediately (non-blocking on a full inbox honors ctx).
func (b *ElevatorBank) Submit(ctx context.Context, req Request) error {
	select {
	case b.inbox <- req:
		return nil
	case <-ctx.Done():
		return ctx.Err()
	}
}

// run is the single coordinator goroutine: one reader, no lock on Assign.
func (b *ElevatorBank) run(ctx context.Context) {
	for {
		select {
		case req := <-b.inbox:
			car := b.scheduler.Assign(req, b.elevators)
			car.enqueue(req) // pushes into car's own channel
		case <-ctx.Done():
			return
		}
	}
}
