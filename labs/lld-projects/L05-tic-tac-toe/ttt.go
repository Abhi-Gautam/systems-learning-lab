package ttt

import "errors"

type Player int8

type Cell = Player

type Move struct {
	Row, Col int
	By       Player
	Seq      int
}

type Status int

const (
	InProgress Status = iota
	Won
	Drawn
)

type Board struct {
	n, k          int
	cells         [][]Cell
	rowCount      [][]int
	colCount      [][]int
	diagCount     []int
	antiDiagCount []int
}

func NewBoard(n, k int, numPlayers int) *Board { return nil }

func (b *Board) At(r, c int) Cell { return b.cells[r][c] }

func (b *Board) Apply(m Move) {
	b.cells[m.Row][m.Col] = Cell(m.By)
	b.rowCount[m.By][m.Row]++
	b.colCount[m.By][m.Col]++
	if m.Row == m.Col {
		b.diagCount[m.By]++
	}
	if m.Row+m.Col == b.n-1 {
		b.antiDiagCount[m.By]++
	}
}

type MoveValidator interface {
	Validate(b *Board, m Move) error
}

type StandardValidator struct{}

func (StandardValidator) Validate(b *Board, m Move) error {
	if m.Row < 0 || m.Row >= b.n || m.Col < 0 || m.Col >= b.n {
		return errors.New("out of bounds")
	}
	if b.cells[m.Row][m.Col] != 0 {
		return errors.New("cell occupied")
	}
	return nil
}

type GravityValidator struct{}

func (GravityValidator) Validate(b *Board, m Move) error { return nil }

type WinChecker interface {
	OnMove(b *Board, m Move) (winner Player, won bool)
}

type NaiveWinChecker struct{}
type IncrementalWinChecker struct{}

func (IncrementalWinChecker) OnMove(b *Board, m Move) (Player, bool) {
	if b.k == b.n {
		if b.rowCount[m.By][m.Row] == b.n ||
			b.colCount[m.By][m.Col] == b.n ||
			(m.Row == m.Col && b.diagCount[m.By] == b.n) ||
			(m.Row+m.Col == b.n-1 && b.antiDiagCount[m.By] == b.n) {
			return m.By, true
		}
		return 0, false
	}
	return windowedCheck(b, m)
}

func windowedCheck(b *Board, m Move) (Player, bool) { return 0, false }

type Game struct {
	board     *Board
	validator MoveValidator
	checker   WinChecker
	players   [2]Player
	turn      int
	seq       int
	status    Status
	winner    Player
}

func (g *Game) Play(m Move) (Status, error) {
	if g.status != InProgress {
		return g.status, errors.New("game over")
	}
	if m.By != g.players[g.turn] {
		return g.status, errors.New("not your turn")
	}
	if err := g.validator.Validate(g.board, m); err != nil {
		return g.status, err
	}
	g.board.Apply(m)
	if w, won := g.checker.OnMove(g.board, m); won {
		g.status, g.winner = Won, w
		return g.status, nil
	}
	g.seq++
	g.turn = 1 - g.turn
	if g.seq == g.board.n*g.board.n {
		g.status = Drawn
	}
	return g.status, nil
}
