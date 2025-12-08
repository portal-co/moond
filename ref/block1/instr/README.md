# Block-1 Instruction Documentation

> Directory: `ref/block1/instr/`
> Source: AGCIS Issue 2 (Machine Instructions) - `ref/moon/agcis_2_machine_instructions.pdf`

## Overview

This directory contains per-instruction documentation for AGC Block-1 architecture instructions. Each file documents a single instruction or instruction variant with modernized C-like pseudocode.

## File Organization

**Naming Convention:** `<opcode>_<variant>.md`
- `<opcode>`: Instruction mnemonic (e.g., `ccs`, `dv`, `mp`, `tc`)
- `<variant>`: Address mode or variant indicator:
  - `k` = K-type (direct address)
  - `e` = E-type (extended)
  - `c` = C-type (counter)
  - `f` = F-type (flow control)
  - `h` = H-type (I/O)

**Examples:**
- `ccs_k.md` - Count, Compare, Skip (K-type)
- `dv_k.md` - Divide (K-type)
- `tc_k.md` - Transfer Control (K-type)
- `pinc.md` - Plus Increment (counter instruction)
- `shanc.md` - Shift and Accumulate (shift instruction)

## Instruction Count

This directory contains **21 Block-1 instruction files**.

## File Structure

Each instruction file follows this standard format:

```markdown
# <INSTRUCTION> — <Description> (modernized)

Source: `agcis_2_machine_instructions.pdf` — pages X–Y (figs., tables).

Summary
- Operation: Brief description
- Behavior: Key characteristics

Micro-op (C-like pseudocode)
<code block with C-like implementation>

Notes
- Edge cases, timing, special behaviors

Audit
- PDF source verification status
- TODO:VERIFY markers if applicable
```

## Style Conventions

### Pseudocode Style (Block-1)
- **Use canonical helper functions** defined in `ref/definitions/`
- Keep functions concise and readable
- Reference shared helpers like `STD2()`, `EXTEND()`, `fetch_instruction()`
- See `ref/definitions/Instruction.md` for type definitions

### Types
- `uint16_t` - 16-bit unsigned word
- `int16_t` - 16-bit signed word
- `(u)int15_t` - 15-bit value in prose (see `ref/cpu/registers.md`)

### Octal Notation
- All octal constants use `0o` prefix (e.g., `0o20`, `0o4000`)

### Memory References
- Use descriptive variable names: `dividend`, `divisor`, `quotient`
- Document register usage explicitly (A, Q, Z, LP, etc.)

## Status and Quality

### Completeness
- All 21 files created and contain pseudocode
- Most files have audit blocks with PDF citations
- Some files contain TODO:VERIFY markers for edge cases

### TODO:VERIFY Markers
Several files contain `TODO:VERIFY` markers for behaviors that need:
- Hardware verification
- Additional PDF sourcing
- Emulation validation

See `ref/TODO_AUDIT.md` for centralized tracking.

## Related Documentation

- **Block-2 Instructions:** `ref/block2/` - Similar instructions for Block-2 architecture
- **CPU Documentation:** `ref/cpu/` - Registers, adder, parity, write amplifiers
- **Canonical Definitions:** `ref/definitions/` - Shared types and helpers (Instruction, STD2, EXTEND)
- **Differences:** Compare with `ref/block2/differences.md` for Block-1 vs Block-2 variations

## Key Instructions

### Arithmetic
- `ad_k.md` - Add
- `su_k.md` - Subtract
- `mp_k.md` - Multiply
- `dv_k.md` - Divide

### Control Flow
- `tc_k.md` - Transfer Control
- `ccs_k.md` - Count, Compare, Skip
- `go.md` - Go (restart)
- `tcsa.md` - Transfer Control to Subaddress

### Indexing & Extension
- `ndx_k.md` - Index
- `xch_k.md` - Exchange

### Counter/Increment
- `pinc.md` - Plus Increment
- `minc.md` - Minus Increment
- `shinc.md` - Shift Increment
- `shanc.md` - Shift Accumulate

### Special
- `ts_k.md` - Transfer to Storage
- `csk_k.md` - Clear and Subtract
- `msk_k.md` - Mask
- `rpt.md` - Repeat
- `rsm.md` - Resume
- `oinc.md` - Overflow Increment
- `linc.md` - Loop Increment

## Source Material

Primary source: **AGCIS Issue 2** (`ref/moon/agcis_2_machine_instructions.pdf`)
- Pages 15–102 (instruction descriptions, tables, timing diagrams)
- Figures 2-1 through 2-35
- Tables 2-1 through 2-5

Supporting sources:
- **AGCIS Issue 3** (`ref/moon/agcis_3_central_processor.pdf`) - CPU behavior, registers
- **AEA Programming Reference** (`ref/moon/AEAProgrammingReference.pdf`) - I/O and scaling

## Usage Notes

When working with these files:
1. Maintain C-like pseudocode style with canonical helpers
2. Add TODO:VERIFY for uncertain behaviors with rationale
3. Update audit blocks when adding PDF citations
4. Keep cross-references to `ref/cpu/registers.md` for types
5. Follow commit convention: `[AI]` prefix with ISO timestamp

---

Last updated: 2025-12-08T04:19:26.568Z
