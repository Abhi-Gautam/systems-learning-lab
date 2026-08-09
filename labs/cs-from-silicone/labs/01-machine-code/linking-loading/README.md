# Linking and Loading — from source to a running process

Platform: Raspberry Pi (Linux aarch64 / ELF). COD §2.12–2.13.

1. One-file pipeline: preprocess → compile → assemble → link → run; name every artifact.
2. Object file guts: ELF header, sections (.text/.data/.bss), symbols, relocations (`readelf`/`nm`).
3. Assembler job: labels → symbol table; pseudoinstructions vs real encoding.
4. Separate compilation: two `.c` → two `.o` → one exe; undef → resolved.
5. Relocation: what the linker patches and why addresses were wrong in isolation.
6. Static library: `.a` via `ar`; which members get pulled; binary size.
7. Shared library: `.so`, `ldd`, `DT_NEEDED`, lazy binding (`LD_DEBUG`).
8. Loader / process image: `/proc/self/maps` — text, libs, stack, heap, entry path.
9. COD §2.13 sort: hand `swap` + `sort` in C; `-O0` asm mapped to book steps.
10. Real library sort: `qsort` / C++ `std::sort`; same inspect path as static/dynamic.
11. Python track: `.pyc` bytecode, `dis`, import as runtime link; contrast with native.

Prerequisite: `function-calls/`.
