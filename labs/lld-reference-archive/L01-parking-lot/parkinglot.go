package parkinglot

import (
	"errors"
	"sync"
	"time"
)

type SpotType int

const (
	SpotCompact SpotType = iota
	SpotRegular
	SpotLarge
	SpotElectric // has charger
)

type VehicleType int

const (
	VehicleMotorcycle VehicleType = iota
	VehicleCar
	VehicleTruck
	VehicleEV
)

type Vehicle struct {
	Plate string
	Type  VehicleType
}

func (v Vehicle) CompatibleSpots() []SpotType {
	switch v.Type {
	case VehicleMotorcycle, VehicleCar:
		return []SpotType{SpotCompact, SpotRegular, SpotLarge}
	case VehicleTruck:
		return []SpotType{SpotRegular, SpotLarge}
	case VehicleEV:
		return []SpotType{SpotCompact, SpotRegular, SpotLarge, SpotElectric}
	default:
		return nil
	}
}

type Spot struct {
	ID    string
	Type  SpotType
	Floor int
	Taken bool
	Plate string
}

type Money int64

type Ticket struct {
	ID        string
	Plate     string
	SpotID    string
	EntryAt   time.Time
	ExitAt    *time.Time
	AmountDue Money
}

type SpotAllocator interface {
	Allocate(v Vehicle) (*Spot, error)
	Release(spotId string) error
}

type PricingPolicy interface {
	Quote(entry, exit time.Time, spotType SpotType) Money
}

type TicketRepository interface {
	Save(ticket *Ticket) error
	Get(id string) (*Ticket, error)
	OpenTickets() ([]*Ticket, error)
}

type Floor struct {
	Level int
	Spots []*Spot
}

type ParkingLot struct {
	mu      sync.Mutex
	name    string
	floors  []*Floor
	alloc   SpotAllocator
	pricing PricingPolicy
	tickets TicketRepository
	clock   func() time.Time
}

func NewParkingLot(name string, floors []*Floor, alloc SpotAllocator, pricing PricingPolicy, tickets TicketRepository) *ParkingLot {
	return &ParkingLot{
		name:    name,
		floors:  floors,
		alloc:   alloc,
		pricing: pricing,
		tickets: tickets,
		clock:   time.Now,
	}
}

var ErrLotFull = errors.New("no compatible spot available")

func (l *ParkingLot) Entry(v Vehicle) (*Ticket, error) {
	l.mu.Lock()
	defer l.mu.Unlock()

	spot, err := l.alloc.Allocate(v)
	if err != nil {
		return nil, err
	}
	ticket := &Ticket{
		ID:      newID(),
		Plate:   v.Plate,
		SpotID:  spot.ID,
		EntryAt: time.Now(),
	}

	if err := l.tickets.Save(ticket); err != nil {
		l.alloc.Release(spot.ID)
		return nil, err
	}
	return ticket, nil
}

func (l *ParkingLot) Exit(ticketId string) (*Ticket, error) {
	l.mu.Lock()
	defer l.mu.Unlock()
}
