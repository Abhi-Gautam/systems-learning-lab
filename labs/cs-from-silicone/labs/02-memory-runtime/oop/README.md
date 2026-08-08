# OOP — from the silicon up

Two runnable, comment-as-lesson files. An object is *bytes in memory*; a method
is a function with a hidden receiver pointer; a virtual/interface call is an
*indirect jump through a function-pointer table*. Everything else
(encapsulation, inheritance, polymorphism, composition) is sugar over that.
Goal: argue OOP mechanics — and the inheritance-vs-composition design fight —
with C++ and Java people on equal footing.

## Concept ladder (both files climb it in this order)

```text
bytes / object layout in memory
→ CPU/ISA view (method = f(this); virtual call = indirect jump via a table)
→ OS/runtime view (vtable, itab, GC, escape)
→ language concept LAST (class, interface, embedding, override)
```

## Run

```bash
cd labs/cs-from-silicone/labs/02-memory-runtime/oop

# C++  (classes, vtables, multiple inheritance, the diamond)
clang++ -std=c++17 -O0 -g -Wall -Wextra oop.cpp -o /tmp/oop && /tmp/oop

# Go   (no classes; structs + implicit interfaces + embedding)
go run oop.go
```

## What each file proves

| Concept | C++ (`oop.cpp`) | Go (`oop.go`) |
|---|---|---|
| object = bytes | member offsets, `this` arithmetic | `unsafe.Sizeof` / `Offsetof`, padding hole |
| encapsulation | `public`/`private`/`protected`; reads a private field via raw `long*` | Exported vs unexported (capitalization), package-scoped |
| construction | ctors/dtors/copy ctor, ordering | `NewT()` convention, zero-value-useful |
| reuse | inheritance + composition, both ways | embedding (method promotion) only |
| polymorphism | **vtable**: `sizeof` grows +8 with first `virtual` | **itab**: interface value is two words → `Sizeof==16` |
| "super" | `Base::method()` | `Outer.Embedded.Method()` |
| hard edges | multiple inheritance, **diamond** (40B dup vs 48B `virtual`) | implicit interface satisfaction, the pointer-receiver gotcha |

## The key cross-language debate

C++ gives you **implementation inheritance** (and pays for it with the fragile
base class, ABI fragility, and the diamond problem — `oop.cpp` prints the
duplicated `A` subobject). Go **refuses inheritance entirely**: reuse is
composition via embedding, polymorphism is *implicit* interface satisfaction,
and interfaces are small and defined by the **consumer**. `oop.go` lays out the
argument with concrete ammunition for either side.

## Learning target

> A class is a struct with a hidden `this`. `virtual` (C++) and interfaces (Go)
> both work the same way underneath — an object carries a pointer to a table of
> function pointers, and a polymorphic call loads one and jumps. C++ bakes the
> table into the object (vptr at offset 0); Go builds it per
> (interface, concrete-type) pair (itab) so types need no declared
> relationship. "Prefer composition over inheritance" isn't taste — it's
> avoiding the coupling and layout problems you can see in these two files.
