package vendingmachine

import (
	"errors"
	"maps"
	"slices"
)

var DefaultAcceptedDenominations = []int{1, 5, 10, 20, 50, 100, 200, 500}

func isAccepted(denom int) bool {
	return slices.Contains(DefaultAcceptedDenominations, denom)
}

var (
	ErrBadDenom          = errors.New("vending: bad denomination")
	ErrUnknownProduct    = errors.New("vending: unknown product")
	ErrOutOfStock        = errors.New("vending: out of stock")
	ErrNoSelection       = errors.New("vending: no selection")
	ErrInsufficientFunds = errors.New("vending: insufficient funds")
	ErrCannotMakeChange  = errors.New("vending: cannot make change")
	ErrNoSession         = errors.New("vending: no session")
)

type Product struct {
	Code  string
	Price int
	Stock int
}

type Session struct {
	Purse     Cash
	Selection string
}

type Machine struct {
	products map[string]Product
	session  *Session
	drawer   Cash
}

type VendResult struct {
	Code   string
	Change Cash
}

func NewMachine(products []Product, drawer Cash) *Machine {
	catalog := make(map[string]Product, len(products))
	for _, p := range products {
		catalog[p.Code] = p
	}
	return &Machine{
		products: catalog,
		drawer:   drawer,
	}
}

func (m *Machine) ensureSession() {
	if m.session == nil {
		m.session = &Session{Purse: Cash{}}
	}
}

func (m *Machine) Insert(denom int) error {
	if !isAccepted(denom) {
		return ErrBadDenom
	}
	m.ensureSession()
	if m.session.Purse == nil {
		m.session.Purse = Cash{}
	}
	m.session.Purse[denom]++
	return nil
}

func (m *Machine) Select(code string) error {
	p, ok := m.products[code]
	if !ok {
		return ErrUnknownProduct
	}
	if p.Stock <= 0 {
		return ErrOutOfStock
	}
	m.ensureSession()
	m.session.Selection = code
	return nil
}

func (m *Machine) Dispense() (VendResult, error) {
	if m.session == nil {
		return VendResult{}, ErrNoSession
	}
	if m.session.Selection == "" {
		return VendResult{}, ErrNoSelection
	}

	code := m.session.Selection
	p, ok := m.products[code]
	if !ok {
		return VendResult{}, ErrUnknownProduct
	}
	if p.Stock <= 0 {
		return VendResult{}, ErrOutOfStock
	}
	credit := m.session.Purse.Total()
	if credit < p.Price {
		return VendResult{}, ErrInsufficientFunds
	}

	available := maps.Clone(m.drawer)
	for d, n := range m.session.Purse {
		available[d] += n
	}

	change, ok := makeChange(available, credit-p.Price)
	if !ok {
		return VendResult{}, ErrCannotMakeChange
	}

	for d, n := range change {
		available[d] -= n
		if available[d] == 0 {
			delete(available, d)
		}
	}

	m.drawer = available
	p.Stock--
	m.products[code] = p
	m.session = nil
	return VendResult{Code: code, Change: change}, nil
}

func (m *Machine) Cancel() (Cash, error) {
	if m.session == nil {
		return Cash{}, nil
	}
	cash := m.session.Purse
	m.session = nil
	return cash, nil
}

func (m *Machine) Credit() int {
	if m.session == nil {
		return 0
	}
	return m.session.Purse.Total()
}

func (m *Machine) Stock(code string) (int, bool) {
	p, ok := m.products[code]
	return p.Stock, ok
}

func (m *Machine) Drawer() Cash { return maps.Clone(m.drawer) }
