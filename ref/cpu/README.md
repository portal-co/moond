# CPU Subsystem Documentation

> Directory: `ref/cpu/`
> Source: AGCIS Issue 3 (Central Processor) - `ref/moon/agcis_3_central_processor.pdf`

## Overview

This directory contains documentation for AGC Central Processor (CP) subsystems and components. These documents describe the hardware behavior that underlies instruction execution.

## Files

### Core Documentation

**`registers.md`** - Register Set & Types (CANONICAL)
- Describes all CP registers: A, Q, Z, LP, B, G, S, SQ
- Defines canonical types: `uint16_t`, `int16_t`, `(u)int15_t`
- Provides helper functions: `read_register()`, `write_register()`, `sign_extend15()`
- **Status:** Authoritative type definitions; referenced by all instruction docs

**`central_processor_overview.md`** - CP Architecture
- High-level overview of the Central Processor
- Control flow and instruction execution pipeline
- Interaction between CP components

**`adder.md`** - Adder & Arithmetic Unit
- 16-bit adder with end-around carry
- Overflow detection and sign handling
- Used by arithmetic instructions (AD, SU, MP, DV)

**`parity_block.md`** - Parity Generation & Checking
- Parity bit generation for words
- Parity checking during memory reads
- Error detection behavior

**`write_amplifiers.md`** - Write Amplifier Logic
- Write gating signals (WG1G through WG6G)
- Address-dependent write behavior
- Shifting and cycling into different bit positions

**`bnk.md`** - Bank & Memory Selection
- Memory bank selection logic
- Addressing modes beyond basic fixed memory
- Interaction with bank registers

## File Count

This directory contains **6 CPU subsystem documentation files**.

## Purpose

These files serve as:
1. **Authoritative references** for hardware behavior
2. **Type definitions** used across all instruction documentation
3. **Helper function specifications** for pseudocode
4. **Cross-reference targets** from instruction files

## Usage by Instruction Docs

Instruction files in `ref/block1/instr/` and `ref/block2/` should:
- Reference `registers.md` for type definitions and register semantics
- Reference `adder.md` for overflow/carry behavior
- Reference `parity_block.md` for parity handling
- Reference `write_amplifiers.md` for special write operations
- Use helper functions defined in these files

## Source Material

Primary source: **AGCIS Issue 3** (`ref/moon/agcis_3_central_processor.pdf`)
- Pages 3-11: Register descriptions and behavior
- Pages 12-20: Adder and arithmetic unit
- Pages 21-25: Parity block and error detection
- Figures 3-1 through 3-8, Tables 3-1 through 3-4

## Canonical Type Definitions

From `registers.md`:
```c
typedef int16_t  int15_t;   // signed 15-bit value
typedef uint16_t uint15_t;  // unsigned 15-bit value

int32_t sign_extend15(uint15_t v);  // Sign extend helper
```

These types are used throughout the instruction documentation to indicate AGC word semantics:
- Bit 0: Parity
- Bits 1-15: Value bits (15-bit magnitude)
- Bit 16: Sign bit (in some representations)

## Register Quick Reference

| Register | Purpose | Size | Usage |
|----------|---------|------|-------|
| **A** | Accumulator | 16-bit | Primary arithmetic register |
| **Q** | Quotient/Return | 16-bit | Return address (TC), quotient (DV) |
| **Z** | Program Counter | 12-bit | Next instruction address |
| **LP** | Low Product | 16-bit | Low-order product (MP, DV) |
| **B** | Next Instruction | 16-bit | Instruction buffer |
| **G** | Memory Buffer | 16-bit | Memory interface |
| **S** | Staging Register | 12-bit | Memory address staging |
| **SQ** | Sequence | 4-bit | Order code for Sequence Generator |

See `registers.md` for complete descriptions.

## Subsystem Interactions

```
Instruction Fetch → B Register → SQ (order code)
                                  ↓
                          Sequence Generator
                                  ↓
                    ┌─────────────┴─────────────┐
                    ↓                           ↓
              Read/Write                     Adder
              G ↔ Memory                   A ± operand
                    ↓                           ↓
              Write Amplifiers           Overflow/Carry
                    ↓                           ↓
              Parity Block                 Update A/Q/Z
```

## Status and Quality

### Completeness
- All core CPU subsystems documented
- Files created during initial AGCIS Issue 3 processing
- Canonical type definitions established in `registers.md`

### TODO:VERIFY Markers
Few TODO:VERIFY markers in CPU docs (most behaviors well-documented in AGCIS Issue 3):
- Some write amplifier gating details
- Specific parity error handling edge cases

See `ref/TODO_AUDIT.md` for tracking.

## Related Documentation

- **Canonical Definitions:** `ref/definitions/` - Instruction types, STD2, EXTEND helpers
- **Block-1 Instructions:** `ref/block1/instr/` - Uses these CPU behaviors
- **Block-2 Instructions:** `ref/block2/` - Uses these CPU behaviors
- **Audit Tracking:** `ref/TODO_AUDIT.md` - TODO:VERIFY markers

## Usage Notes

When documenting instructions:
1. Reference `registers.md` for types: "See `ref/cpu/registers.md` for type definitions"
2. Use canonical helper names: `read_register()`, `write_register()`, `sign_extend15()`
3. Reference specific CPU docs for behavior: "Overflow detected per `ref/cpu/adder.md`"
4. Keep CPU docs as authoritative sources; don't duplicate content in instruction files

---

Last updated: 2025-12-08T04:19:26.568Z
