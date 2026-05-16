# N2T - Nand2Tetris
## Doubts & Clarifications

---

## [2026-01-21] Chapter 4 Grand Challenge: Master Hack Machine Language

### Overview

This challenge covers ALL of Chapter 4 through three progressive exercises:

```
┌─────────────────────────────────────────────────────────────────┐
│  LEVEL 1: Mult.asm        │  Loops, variables, arithmetic      │
├─────────────────────────────────────────────────────────────────┤
│  LEVEL 2: Fill.asm        │  Keyboard input, screen output     │
├─────────────────────────────────────────────────────────────────┤
│  LEVEL 3: Diagonal.asm    │  Bit manipulation, advanced logic  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Prerequisites: Hack Machine Language Reference

### A-Instruction (Address/Value Loading)

```
Syntax:  @value   or   @symbol

Binary:  0 v v v v v v v v v v v v v v v
         │ └─────────────┬─────────────┘
         │          15-bit value
         └── 0 = A-instruction

Effect:  A register ← value

Examples:
  @21      // A = 21
  @SCREEN  // A = 16384 (predefined symbol)
  @R5      // A = 5 (R5 is alias for address 5)
  @myvar   // A = address assigned to 'myvar'
```

### C-Instruction (Compute)

```
Syntax:  dest = comp ; jump   (dest and jump are optional)

Binary:  1 1 1 a c c c c c c d d d j j j
         │     │ └─────┬─────┘ └─┬─┘ └─┬─┘
         │     │    comp bits  dest  jump
         │     └── a=0: use A, a=1: use M
         └── 111 = C-instruction

dest options: null, M, D, MD, A, AM, AD, AMD
comp options: 0, 1, -1, D, A, M, !D, !A, !M, -D, -A, -M,
              D+1, A+1, M+1, D-1, A-1, M-1,
              D+A, D+M, D-A, D-M, A-D, M-D,
              D&A, D&M, D|A, D|M
jump options: null, JGT, JEQ, JGE, JLT, JNE, JLE, JMP
```

### Predefined Symbols

```
┌──────────────┬─────────┬────────────────────────────┐
│ Symbol       │ Value   │ Purpose                    │
├──────────────┼─────────┼────────────────────────────┤
│ R0-R15       │ 0-15    │ Virtual registers          │
│ SP           │ 0       │ Stack pointer              │
│ LCL          │ 1       │ Local segment base         │
│ ARG          │ 2       │ Argument segment base      │
│ THIS         │ 3       │ This segment base          │
│ THAT         │ 4       │ That segment base          │
│ SCREEN       │ 16384   │ Screen memory map base     │
│ KBD          │ 24576   │ Keyboard memory map        │
└──────────────┴─────────┴────────────────────────────┘
```

### Memory Map

```
┌─────────────────────────────────────┐ 0x0000 (0)
│         RAM (16K words)             │
│   R0-R15: addresses 0-15            │
│   Static: addresses 16-255          │
│   Stack:  addresses 256-2047        │
│   Heap:   addresses 2048-16383      │
├─────────────────────────────────────┤ 0x4000 (16384)
│         Screen (8K words)           │
│   256 rows × 32 words per row       │
│   512 pixels wide × 256 pixels tall │
├─────────────────────────────────────┤ 0x6000 (24576)
│         Keyboard (1 word)           │
│   Contains scan code of pressed key │
│   0 if no key pressed               │
└─────────────────────────────────────┘
```

---

## LEVEL 1: Mult.asm (Official Project 4a)

### Specification

```
Multiply RAM[0] by RAM[1] and store result in RAM[2].

Inputs:
  R0 = first number (assume R0 >= 0)
  R1 = second number (assume R1 >= 0)

Output:
  R2 = R0 × R1

Constraint: Hack has NO multiplication instruction!
```

### The Insight

```
Multiplication = Repeated Addition

5 × 3 = 5 + 5 + 5 = 15

Algorithm:
  result = 0
  counter = R1
  while counter > 0:
      result = result + R0
      counter = counter - 1
  R2 = result
```

### Skeleton Code

```asm
// Mult.asm: Computes R2 = R0 * R1
// Strategy: Add R0 to itself R1 times

    // Initialize R2 = 0
    @R2
    M=0

    // Load counter = R1
    @R1
    D=M
    @counter
    M=D

(LOOP)
    // if counter <= 0, goto END
    @counter
    D=M
    @END
    D;JLE

    // R2 = R2 + R0
    // TODO: Your code here

    // counter = counter - 1
    // TODO: Your code here

    // goto LOOP
    @LOOP
    0;JMP

(END)
    @END
    0;JMP
```

### Test Cases

```
Test 1: R0=3, R1=5  → R2 should be 15
Test 2: R0=0, R1=7  → R2 should be 0
Test 3: R0=12, R1=0 → R2 should be 0
Test 4: R0=1, R1=1  → R2 should be 1
```

### Skills Tested
- [x] Variables with @symbol
- [x] Loops with labels and JMP
- [x] Conditionals with comparison jumps
- [x] Reading/writing RAM (M=, D=M)
- [x] Arithmetic (D+M, M-1)

**Status: [ ] NOT STARTED  [ ] IN PROGRESS  [ ] COMPLETE**

---

## LEVEL 2: Fill.asm (Official Project 4b)

### Specification

```
Infinite loop that:
- When ANY key is pressed → fill entire screen BLACK (all 1s)
- When NO key is pressed  → fill entire screen WHITE (all 0s)

Must respond continuously (not just once).
```

### Memory Details

```
Keyboard (KBD = 24576):
┌─────────────────────────────────────┐
│ RAM[24576] = scan code of key       │
│              0 if no key pressed    │
└─────────────────────────────────────┘

Screen (SCREEN = 16384):
┌─────────────────────────────────────┐
│ 8192 consecutive 16-bit words       │
│ RAM[16384] to RAM[24575]            │
│ Set word to -1 (0xFFFF) = 16 black  │
│ Set word to 0 = 16 white pixels     │
└─────────────────────────────────────┘
```

### The Algorithm

```
MAIN_LOOP:
    // Check keyboard
    if RAM[KBD] == 0:
        color = 0      // white
    else:
        color = -1     // black (all 1s = 0xFFFF)

    // Fill screen with color
    for addr = SCREEN to SCREEN+8191:
        RAM[addr] = color

    goto MAIN_LOOP
```

### Skeleton Code

```asm
// Fill.asm: Fill screen based on keyboard input

(MAIN)
    // Read keyboard
    @KBD
    D=M

    // if key pressed (D != 0), set color to black
    @BLACK
    D;JNE

    // else set color to white
    @color
    M=0
    @FILL
    0;JMP

(BLACK)
    @color
    M=-1       // -1 in two's complement = 0xFFFF = all 1s

(FILL)
    // Initialize screen pointer
    @SCREEN
    D=A
    @addr
    M=D

    // Calculate end address (SCREEN + 8192)
    @8192
    D=A
    @SCREEN
    D=D+A
    @end
    M=D

(FILL_LOOP)
    // if addr >= end, goto MAIN
    @addr
    D=M
    @end
    D=D-M
    @MAIN
    D;JGE

    // RAM[addr] = color
    // TODO: Your code here
    // Hint: Need to use A as pointer

    // addr = addr + 1
    // TODO: Your code here

    // continue loop
    @FILL_LOOP
    0;JMP
```

### The Tricky Part: Pointer Dereferencing

```asm
// To write to RAM[addr] where addr is a variable:

@color
D=M        // D = color value

@addr
A=M        // A = value stored in addr (the pointer!)
M=D        // RAM[A] = D  →  RAM[addr] = color

// This is INDIRECT ADDRESSING
// addr holds an address, we use that address
```

### Test Cases

```
1. Run program
2. Press any key → screen should turn black
3. Release key → screen should turn white
4. Hold different keys → should stay black
5. Must respond continuously (not freeze)
```

### Skills Tested
- [x] Keyboard memory-mapped input
- [x] Screen memory-mapped output
- [x] Indirect addressing (pointers)
- [x] Nested loops
- [x] Continuous program (infinite loop)
- [x] Two's complement (-1 = all 1s)

**Status: [ ] NOT STARTED  [ ] IN PROGRESS  [ ] COMPLETE**

---

## LEVEL 3: Diagonal.asm (Bonus Challenge)

### Specification

```
Draw a diagonal line from top-left (0,0) to bottom-right area.
Line goes from pixel (0,0) to approximately (255,255).

    ■
     ■
      ■
       ■
        ■
         .
          .
           .
```

### Why This Is Hard

```
Challenge 1: Screen addressing
  pixel(row, col) → RAM[16384 + row×32 + col/16], bit (col%16)

Challenge 2: No multiplication
  row × 32 must use repeated addition or shifts

Challenge 3: No division
  col / 16 and col % 16 must be computed manually

Challenge 4: Bit manipulation
  Must set ONE bit without affecting others
  Need to compute bit mask: 2^(col%16)
  Need to OR with existing value
```

### Solving Each Challenge

#### Challenge 2: Multiply by 32 (shift left 5)

```asm
// temp = row × 32

@row
D=M
@temp
M=D        // temp = row

// Double 5 times: temp = temp × 2 × 2 × 2 × 2 × 2
@temp
D=M
M=D+M      // temp = temp × 2
D=M
M=D+M      // temp = temp × 4
D=M
M=D+M      // temp = temp × 8
D=M
M=D+M      // temp = temp × 16
D=M
M=D+M      // temp = temp × 32
```

#### Challenge 3: Divide by 16 and Mod 16

```asm
// For diagonal: col = row, so col/16 = row/16

// Approach: Subtract 16 repeatedly, count iterations
// quotient = col / 16
// remainder = col % 16

@col
D=M
@remainder
M=D        // remainder = col
@quotient
M=0        // quotient = 0

(DIV_LOOP)
    @remainder
    D=M
    @16
    D=D-A
    @DIV_DONE
    D;JLT      // if remainder < 16, done

    @remainder
    M=D        // remainder = remainder - 16
    @quotient
    M=M+1      // quotient++
    @DIV_LOOP
    0;JMP

(DIV_DONE)
    // quotient = col/16, remainder = col%16
```

#### Challenge 4: Compute Bit Mask (2^remainder)

```asm
// mask = 1 << remainder = 2^remainder

@mask
M=1        // mask = 1

@remainder
D=M
@MASK_DONE
D;JEQ      // if remainder == 0, mask = 1, done

@shift_count
M=D        // shift_count = remainder

(SHIFT_LOOP)
    @mask
    D=M
    M=D+M      // mask = mask × 2 (left shift)

    @shift_count
    MD=M-1     // shift_count--, D = new value
    @SHIFT_LOOP
    D;JGT      // if shift_count > 0, continue

(MASK_DONE)
    // mask now contains 2^remainder
```

#### Challenge 5: Set Single Bit with OR

```asm
// RAM[screen_addr] = RAM[screen_addr] | mask

@screen_addr
A=M        // A = screen_addr value
D=M        // D = current screen word

@mask
D=D|M      // D = D OR mask

@screen_addr
A=M        // A = screen_addr again
M=D        // write back
```

### Full Algorithm Pseudocode

```
row = 0

LOOP:
    col = row  (diagonal: col equals row)

    // Calculate word address
    word_offset = row × 32 + col / 16
    screen_addr = 16384 + word_offset

    // Calculate bit position and mask
    bit_pos = col % 16
    mask = 2^bit_pos

    // Set the pixel
    RAM[screen_addr] = RAM[screen_addr] | mask

    // Next pixel
    row = row + 1

    // Check bounds
    if row < 256:
        goto LOOP

END:
    infinite loop
```

### Verification Points

```
Check these pixels are drawn correctly:

(row, col)  → RAM address, bit position
───────────────────────────────────────
(0, 0)      → RAM[16384], bit 0
(1, 1)      → RAM[16416], bit 1
(15, 15)    → RAM[16864], bit 15
(16, 16)    → RAM[16897], bit 0   ← word changes!
(31, 31)    → RAM[17377], bit 15
(32, 32)    → RAM[17410], bit 0
(255, 255)  → RAM[24544], bit 15  ← last pixel
```

### Skills Tested
- [x] All skills from Level 1 and 2
- [x] Implementing multiplication with shifts
- [x] Implementing division/modulo
- [x] Bit mask generation
- [x] Bitwise OR for selective bit setting
- [x] Complex address calculation
- [x] Multiple nested computations

**Status: [ ] NOT STARTED  [ ] IN PROGRESS  [ ] COMPLETE**

---

## Connection to Your Hardware (Full Circle!)

```
When you run M=D|M in Fill.asm:

YOUR ASSEMBLY          YOUR HARDWARE (Chapters 1-3)
──────────────────────────────────────────────────────

@addr                  → A-Register loads value
A=M                    → RAM[A] outputs to A-Register
D=M                    → RAM[A] outputs to D-Register
@mask
D=D|M   ──────────────→ ALU: comp=D|M
                         │
                         │  zx=0, nx=1, zy=0, ny=1
                         │  f=0, no=1
                         │  (De Morgan: !(!D & !M) = D|M)
                         │
                         ▼
@addr                  → A-Register
A=M                    → Address decoder → select register
M=D    ──────────────→ load=1 for that RAM register
                         │
                         ▼
                       Clock tick! DFFs capture value
                         │
                         ▼
                       Screen hardware reads new bits
                         │
                         ▼
                       Pixels appear!
```

---

## Checklist: Chapter 4 Complete Understanding

After completing all three levels, verify you understand:

**A-Instruction:**
- [ ] How @value loads into A register
- [ ] Difference between @5, @R5, @SCREEN, @myvar
- [ ] How symbols get resolved to addresses

**C-Instruction:**
- [ ] All comp operations and their binary encoding
- [ ] How dest determines where result goes
- [ ] How jump conditions work with ALU output flags

**Memory-Mapped I/O:**
- [ ] Screen address calculation formula
- [ ] How keyboard input is read
- [ ] Why writing to RAM changes the display

**Programming Patterns:**
- [ ] Loops with labels and jumps
- [ ] Conditionals with comparison jumps
- [ ] Indirect addressing (pointers)
- [ ] Arithmetic without MUL/DIV instructions

**Hardware Connection:**
- [ ] How assembly maps to your ALU operations
- [ ] How RAM addressing uses your DMUX/MUX
- [ ] How clock edges trigger your DFFs

---
