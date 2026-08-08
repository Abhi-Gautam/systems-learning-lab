# Pointers — from the silicon up

Two runnable, comment-as-lesson files. Read the comments top-to-bottom; the
printed output is the proof. Goal: understand an address as a *number naming a
byte cell*, then build `&`, `*`, references, smart pointers, stack/heap, and
the cache cost of pointer-chasing on top of it — well enough to argue layout
with anyone.

## Concept ladder (both files climb it in this order)

```text
physical byte cell at a numeric address
→ CPU/ISA view (an address is a number in a register; LOAD / STORE)
→ OS/runtime view (stack frame, heap region, virtual memory, GC)
→ language concept LAST (pointer, reference, smart pointer, escape)
```

## Run

```bash
cd labs/cs-from-silicone/labs/02-memory-runtime/pointers

# C++  (manual lifetime, pointer arithmetic, RAII, smart pointers)
clang++ -std=c++17 -O0 -g -Wall -Wextra pointers.cpp -o /tmp/pointers && /tmp/pointers

# Go   (no arithmetic, escape analysis, GC)
go run pointers.go
go build -gcflags=-m pointers.go   # watch the compiler decide stack vs heap
rm -f pointers                     # remove the build artifact
```

## What each file proves

| Concept | C++ (`pointers.cpp`) | Go (`pointers.go`) |
|---|---|---|
| address = number | prints `&x`, neighbors, `sizeof(ptr)==8` | same, hex addresses |
| `&` / `*` | `*p = 5` mutates `x` | same; nil-deref panic caught via `recover()` |
| pass semantics | value vs `T*` vs `T&` | value-only; `*T` to opt into mutation |
| pointer→pointer | `**pp`, `***` chain | `**int` chain |
| layout cost | `const` flavors, pointer arithmetic, array decay | **no arithmetic by design**; slice/map gotcha |
| stack vs heap | **you** decide (`new`/`delete`) | **compiler** decides (escape analysis) |
| lifetime | RAII, `unique`/`shared`/`weak_ptr`, cycle leak | GC, no `free` |
| cache climax | array scan vs shuffled list: **~42× slower** | same: **~200× slower** |

## The key cross-language debate

Returning `&local` is a **dangling-pointer bug in C++** (the stack frame is
gone) but **safe in Go** — escape analysis moves it to the heap and the GC
keeps it alive. `pointers.go` makes the compiler print `moved to heap: local`
so you see the decision happen. That single contrast is most of the
C-vs-managed-language argument.

## Learning target

> A pointer is not a language feature — it is a number that names a byte. `&`
> reads that number, `*` follows it. A copy is a copy because it lives at a
> *different* address. Linked lists lose to arrays not by doing more work but
> by scattering bytes across cache lines. C hands you the address and the
> lifetime; Go hands you the address and hides the lifetime.
