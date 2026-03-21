# moond

An AGC (Apollo Guidance Computer) Block-2 emulator and recompiler, written in Rust with a C reference implementation.

This is an active, early-stage project. The core infrastructure is functional — the interpreter executes instructions, the assembler/disassembler round-trips correctly, and the recompiler produces both C and WebAssembly output — but it is not yet a complete, end-to-end runnable AGC simulation (no I/O peripherals, no task scheduler, no rope memory loader for real AGC binaries).

## What it actually does

The project implements the AGC Block-2 instruction set at multiple levels:

1. **Interpreter** — a fetch/decode/execute loop that runs AGC machine code word by word, tracking EXTEND state, ones-complement arithmetic, bank-switched memory, and I/O channels.
2. **Assembler / disassembler** — assembles mnemonic text to 15-bit words and disassembles in the other direction, with EXTEND-state tracking.
3. **Recompiler** — translates AGC binary programs to C source code or WebAssembly via a two-stage pipeline: a frontend that builds a basic-block IR, and backends that emit C (via computed-goto dispatch) or WASM (via the `yecta` reactor framework).
4. **C reference implementation** — a parallel C implementation (`src/core.c`, `include/core.h`) of the Block-2 memory bank model, linked into Rust via `bindgen` for cross-testing.

## Architecture

```
┌──────────────────────────────────────────────────────────────────────┐
│  agc-isa          Block-2 ISA table, opcode formats, semantics DSL   │
│                   (no_std; no external deps)                          │
├──────────────────────────────────────────────────────────────────────┤
│  agc-interp       Fetch/decode/execute interpreter (no_std)          │
│                   CPU state: A, L, Q, EB, FB, Z, BB + memory + I/O  │
├──────────────────────────────────────────────────────────────────────┤
│  agc-asm          Assembler + disassembler                           │
├──────────────────────────────────────────────────────────────────────┤
│  agc-driver       Loads ISA specs from markdown files on disk        │
├──────────────────────────────────────────────────────────────────────┤
│  agc-lower        AGC SemOp → TC2 16-bit stack bytecode lowering     │
├──────────────────────────────────────────────────────────────────────┤
│  agc-recompile    Recompiler: frontend (IR) + C backend + WASM backend│
├──────────────────────────────────────────────────────────────────────┤
│  moond-sys        Rust FFI bindings to the C implementation          │
├──────────────────────────────────────────────────────────────────────┤
│  agc-crosstest    Tests: exhaustively verifies C and Rust decoders   │
│                   agree across all 65536 × 2 (word, extend) pairs    │
└──────────────────────────────────────────────────────────────────────┘
```

## Crates

### `agc-isa`
Defines the ISA: `InstrType` enum (all ~50 Block-2 instructions), `InstrSpec` (mnemonic, opcode format, addressing mode, semantics), and a builtin spec table with semantics expressed in a small DSL. `no_std`. No external dependencies.

### `agc-interp`
The reference interpreter. `Cpu` struct holds all registers (`A`, `L`, `Q`, `EB`, `FB`, `Z`, `BB`), 1024-word erasable memory, 6144-word fixed memory, and 512 I/O channels. Arithmetic uses 15-bit ones-complement with end-around carry. EXTEND flip-flop is tracked per instruction.

### `agc-asm`
Assembles a single line of AGC assembly text to one or two 15-bit words (two for extracode instructions that require EXTEND prefix). Disassembles a slice of words to text, tracking EXTEND state across the stream.

### `agc-driver`
Loads ISA semantics from a directory of markdown files. Each `.md` file can contain a `## Semantics` section with an `agc-sem` code block; the loader parses these and overlays them on the builtin spec table.

### `agc-lower`
Lowers AGC `SemOp` semantic sequences to TC2 (two's-complement) 16-bit stack bytecode. Ones-complement operations are emitted as dedicated VM opcodes that call the same arithmetic routines as the interpreter, ensuring bit-exact results. Includes a `BytecodeVm` for executing the lowered code.

### `agc-recompile`
Two-stage recompiler:
- **Frontend** (`frontend.rs`): recursive-descent decode over a 4096-word AGC memory image. Builds a `BTreeMap<u16, BasicBlock>` IR. Tracks EXTEND state. Handles `NDX` (index) constant-folding where possible, falling back to `Terminator::Indirect` when the operand is in erasable memory.
- **C backend** (`backend/c.rs`): emits a C translation unit. Each basic block becomes a labeled C block; branches become `goto`s. Uses a local `_stk[]` array for the TC2 evaluation stack and a runtime header for memory/channel dispatch.
- **WASM backend** (`backend/wasm.rs`): emits a WebAssembly module via the `yecta` reactor. Produces 8192 functions (4096 addresses × 2 EXTEND states). Register file is mapped into a 64 KB WASM memory page. Memory and channel access are imported functions.

### `moond-sys`
`no_std` Rust bindings to the C implementation, generated at build time by `bindgen`. The C sources are copied in via `sync.sh` / `cc-copy.sh` from the workspace root. Used by `agc-crosstest`.

### `agc-crosstest`
Cross-validation tests. The main test sweeps all 65536 possible 15-bit words with both `extend=false` and `extend=true`, running both the C and Rust decoders and asserting they agree on the mnemonic and decoded address. Hardware-triggered instructions (PINC, MINC, etc.) are excluded since they have no opcode encoding.

## Memory model

The AGC Block-2 has a 12-bit address space (0o0000–0o7777):

| Range | Description |
|---|---|
| 0o0000–0o0007 | Central registers (A, L, Q, EB, FB, Z, BB, ZERO) |
| 0o0010–0o1777 | Erasable memory, bank-switched via EB |
| 0o2000–0o3777 | Fixed-fixed memory |
| 0o4000–0o7777 | Fixed memory, bank-switched via FB and superbank bits |

Words are 15 bits. The accumulator (A) carries a 16th overflow bit. Arithmetic is ones-complement: minus-zero (0o77777) is a distinct value from plus-zero.

## Reference material

The `ref/moon/` directory contains original AGC documentation:
- `agcis_2_machine_instructions.pdf`
- `agcis_3_central_processor.pdf`
- `agcis_32_blk2_instructions.pdf`
- `agc4_memo9_rev_june1967.pdf`
- `AEAProgrammingReference.pdf`

## Building

Requires Rust (edition 2021/2024), a C compiler (`cc`) on PATH, and `clang`/`libclang` for `bindgen`. The `moond-sys` build script runs `sync.sh` to copy C sources before compilation.

```sh
cargo build
cargo test
```

The `agc-recompile` e2e tests invoke the system C compiler (`cc -fsyntax-only`) to validate generated C output. The `agc-crosstest` tests link the C implementation; both require a working C toolchain.

## Status

- ISA table: complete for all Block-2 instructions with semantics
- Interpreter: working; unit tests cover CA, AD, TC, CCS, XCH, EXTEND, and ones-complement arithmetic
- Assembler/disassembler: working; round-trip tested
- C/Rust cross-decoder agreement: tested exhaustively across all word encodings
- Recompiler C backend: working; tested with TCF loops, CA+TC sequences, CCS four-way branches, NDX
- Recompiler WASM backend: working; output validated with `wasmparser`
- I/O peripherals: not implemented
- AGC task scheduler / rope memory loader: not implemented
- Running real AGC flight software: not attempted

## License

Dual-licensed: GNU AGPLv3 or a proprietary license from Portal Solutions LLC. See `COPYING.md`.
