## LLD lab projects

One Go module per LLD design problem from `notes/lld-notes.md`. Each project hosts the **Code skeleton** that used to live inline in the note, so notes stay prose-and-diagrams and the code stays runnable.

| Slug | Note | Pattern spine |
|---|---|---|
| `L01-parking-lot` | L01 · Parking lot | Two Strategy seams: `SpotAllocator` × `PricingPolicy`; `ParkingLot` aggregate root |
| `L02-elevator` | L02 · Elevator bank scheduler | Strategy `Scheduler` + per-car `State`; single coordinator goroutine |
| `L04-vending-machine` | L04 · Vending machine | `State` over txn lifecycle; Strategy `ChangeMaker` (greedy / DP) |
| `L05-tic-tac-toe` | L05 · Tic-Tac-Toe (N×N) | Strategy `MoveValidator` × `WinChecker`; incremental row/col/diag counters |
| `L10-splitwise` | L10 · Splitwise expense splitter | Event-sourced `Expense` log; `SplitStrategy`, `Simplifier` seams |

Each project is a standalone Go module — `cd labs/lld-projects/L0X-... && go build ./...` works in isolation. Stub bodies are intentional; bodies live in the corresponding Deep dive in the note.
