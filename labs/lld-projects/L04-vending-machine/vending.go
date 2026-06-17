package vending

type Coin struct{ ValueCents int }
type Denomination int
type ItemID string

type Slot struct {
	PriceCents int
	Stock      int
}

type Inventory struct{ slots map[ItemID]Slot }

func (i *Inventory) Reserve(id ItemID) error { return nil }
func (i *Inventory) Commit(id ItemID)        {}
func (i *Inventory) Release(id ItemID)       {}

type State interface {
	OnCoin(*VendingMachine, Coin)
	OnSelect(*VendingMachine, ItemID)
	OnDispense(*VendingMachine)
	OnCancel(*VendingMachine)
}

type VendingMachine struct {
	state        State
	balanceCents int
	selectedID   ItemID
	inventory    *Inventory
	changeMaker  ChangeMaker
	ledger       Ledger
	available    []Denomination
}

func New(inv *Inventory, cm ChangeMaker) *VendingMachine {
	m := &VendingMachine{inventory: inv, changeMaker: cm}
	m.state = &IdleState{}
	return m
}

func (m *VendingMachine) InsertCoin(c Coin)    { m.state.OnCoin(m, c) }
func (m *VendingMachine) SelectItem(id ItemID) { m.state.OnSelect(m, id) }
func (m *VendingMachine) Dispense()            { m.state.OnDispense(m) }
func (m *VendingMachine) Cancel()              { m.state.OnCancel(m) }

type IdleState struct{}

func (s *IdleState) OnCoin(m *VendingMachine, c Coin) {
	m.balanceCents += c.ValueCents
	m.ledger.Append("coin", m.balanceCents)
	m.state = &CoinAcceptedState{}
}
func (s *IdleState) OnSelect(*VendingMachine, ItemID) {}
func (s *IdleState) OnDispense(*VendingMachine)       {}
func (s *IdleState) OnCancel(*VendingMachine)         {}

type CoinAcceptedState struct{}

func (s *CoinAcceptedState) OnCoin(m *VendingMachine, c Coin) {
	m.balanceCents += c.ValueCents
}
func (s *CoinAcceptedState) OnSelect(m *VendingMachine, id ItemID) {
	slot, ok := m.inventory.slots[id]
	if !ok || slot.Stock == 0 || m.balanceCents < slot.PriceCents {
		return
	}
	_ = m.inventory.Reserve(id)
	m.selectedID = id
	m.state = &ItemSelectedState{}
}
func (s *CoinAcceptedState) OnDispense(*VendingMachine) {}
func (s *CoinAcceptedState) OnCancel(m *VendingMachine) {
	refund, _ := m.changeMaker.Make(m.balanceCents, m.available)
	_ = refund
	m.balanceCents = 0
	m.state = &IdleState{}
}

type ItemSelectedState struct{}

func (s *ItemSelectedState) OnCoin(*VendingMachine, Coin)     {}
func (s *ItemSelectedState) OnSelect(*VendingMachine, ItemID) {}
func (s *ItemSelectedState) OnDispense(m *VendingMachine) {
	m.inventory.Commit(m.selectedID)
	m.state = &DispensingState{}
}
func (s *ItemSelectedState) OnCancel(m *VendingMachine) {
	m.inventory.Release(m.selectedID)
	refund, _ := m.changeMaker.Make(m.balanceCents, m.available)
	_ = refund
	m.balanceCents = 0
	m.state = &IdleState{}
}

type DispensingState struct{}

func (s *DispensingState) OnCoin(*VendingMachine, Coin)     {}
func (s *DispensingState) OnSelect(*VendingMachine, ItemID) {}
func (s *DispensingState) OnDispense(m *VendingMachine) {
	m.balanceCents = 0
	m.state = &IdleState{}
}
func (s *DispensingState) OnCancel(*VendingMachine) {}

type ChangeMaker interface {
	Make(amountCents int, avail []Denomination) ([]Coin, error)
}

type GreedyChangeMaker struct{}
type DPChangeMaker struct{}

func (g *GreedyChangeMaker) Make(amt int, d []Denomination) ([]Coin, error)  { return nil, nil }
func (d *DPChangeMaker) Make(amt int, ds []Denomination) ([]Coin, error)     { return nil, nil }

type Ledger interface{ Append(event string, balance int) }
