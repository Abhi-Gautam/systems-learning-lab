package tictactoe

type Player struct {
	id    int
	name  string
	moves []*Pos
}

type Pos struct {
	x int
	y int
}

const (
	BoardSize  = 3
	MovesToWin = 3
)

type MoveStatus int

const (
	StatusSuccess MoveStatus = iota
	StatusInvalid
	StatusRetry
	StatusWin
	StatusLose
	StatusDraw
	StatusDone
)

func StartNewGame(p1, p2 Player) {
	// Initialize the 2 players
	// Start the engine session
}

func (p *Player) Move(pos Pos) MoveStatus {
	// Talks to the engine
	// Returns what the status after this move
	// Win, lose, inalid, retry, draw, done
	return StatusSuccess
}

type Board struct {
}

type Engine struct {
	board         *Board
	currentPlayer *Player
	status        MoveStatus
}

func NewEngine() *Engine {
	return &Engine{}
}

func (e *Engine) move(player *Player, pos Pos) MoveStatus {
	// Check current player turn
	// Check board status and validate move
	// Update board and player state
	// Return the status of the move
	return StatusSuccess
}
