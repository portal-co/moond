# Block-2 Documentation

> Directory: `ref/block2/`
> Source: AGCIS Issue 32 (Block-2 Instructions) - `ref/moon/agcis_32_blk2_instructions.pdf`

## Overview

This directory contains per-instruction documentation for AGC Block-2 architecture. Block-2 is an enhanced version of the AGC with additional instructions, modified timing, and extended addressing modes.

## File Organization

**Naming Convention:** `<opcode>_<variant>.md`
- Same convention as Block-1, but with Block-2-specific behaviors
- Variants: `k`, `e`, `c`, `f`, `h` (see Block-1 README for meanings)

**Examples:**
- `ccs_e.md` - Count, Compare, Skip (E-type extended)
- `dv_e.md` - Divide (E-type extended)
- `ad_k.md` - Add (K-type, may differ from Block-1)
- `inotld_h.md` - I/O Not Load (Block-2 specific)

## Instruction Count

This directory contains **41 Block-2 instruction files**.

## Block-2 vs Block-1 Differences

### Key Architectural Differences
- **Extended memory:** E-type instructions for addressing beyond basic memory
- **Additional I/O:** New peripheral and counter instructions (INOTLD, INOTRD, etc.)
- **Modified timing:** Some instructions have different subinstruction sequences
- **Counter enhancements:** Additional counter control instructions (PCDU, MCDU, etc.)

See `ref/block2/differences.md` for detailed behavioral divergences.

## File Structure

Each Block-2 instruction file follows this format:

```markdown
# <INSTRUCTION> — <Description> (Block-2)

Summary
- Operation: Brief description
- Block-2 specifics: Differences from Block-1 if applicable

Modernized pseudocode
<code block with C-like implementation>

Notes
- Subinstructions may be inlined for Block-2 with "Inline notes"
- Edge cases and timing differences

Audit
- PDF source verification status
- TODO:VERIFY markers if applicable
```

## Style Conventions

### Pseudocode Style (Block-2)
- **Inline small helpers** when it improves clarity
- Add "Inline notes" block explaining why inlined (timing, fusion)
- Comment inlined code with reference to canonical helper
- See `ref/definitions/` for canonical definitions

Example:
```c
// Inline STMIC_stage() for Block-2 timing clarity
// (See ref/definitions/STD2.md for canonical version)
uint16_t z = Z;
S = z; Y = z; X = 0;
// ... inlined code ...
```

### Types and Notation
- Same as Block-1: `uint16_t`, `int16_t`, `0o` prefix for octal
- See `ref/cpu/registers.md` for canonical type definitions

## Status and Quality

### Completeness
- 41 instruction files created during 2025-12-07 documentation pass
- All files have basic structure and pseudocode
- Most files have audit blocks with AGCIS Issue 2/3 citations
- Some files are stubs needing expanded pseudocode

### TODO:VERIFY Markers
Many Block-2 files contain `TODO:VERIFY` markers for:
- E-memory restore timing and behavior
- EXT (extended) instruction sequencing
- SCALER channel width alignment
- Overflow bit propagation details

See `ref/TODO_AUDIT.md` for centralized tracking of all 76 TODO:VERIFY markers.

## Directory Structure

```
block2/
├── README.md (this file)
├── differences.md (Block-2 vs Block-1 summary)
├── index.md (instruction listing)
├── definitions/ (Block-2-specific definitions if needed)
├── *.md (41 instruction files)
└── (pseudocode stubs archived to ref/_archive/pseudocode_stubs/)
```

## Major Instruction Categories

### Arithmetic & Logic (E-type Extended)
- `ad_k.md`, `ads_e.md` - Add
- `su_e.md`, `msu_e.md` - Subtract
- `mp_k.md` - Multiply
- `dv_e.md`, `das_e.md` - Divide
- `aug_e.md` - Augment
- `dim_e.md` - Diminish
- `incr_e.md` - Increment

### Control Flow
- `tc_k.md` - Transfer Control
- `ccs_e.md` - Count, Compare, Skip (E-type)
- `tcf_f.md` - Transfer Control to Fixed
- `bzf_f.md` - Branch Zero to Fixed
- `tcsaj_k.md` - Transfer Control to Subaddress and Jump
- `go.md` - Go (restart)
- `rupt.md` - Interrupt handling

### Memory & Exchange
- `xch_e.md` - Exchange (E-type)
- `store_e.md` - Store
- `fetch_k.md` - Fetch
- `ca_k.md` - Clear and Add

### Indexing & Extension
- `ndx_e.md` - Index (E-type)
- `resume.md` - Resume from interrupt

### Counter & Peripheral (Block-2 Enhanced)
- `pinc_c.md`, `minc_c.md` - Increment counters
- `shinc_c.md`, `shanc_c.md` - Shift and increment
- `dinc_c.md` - Double increment
- `pcdu_c.md` - Plus Counter Down Up
- `mcdu_c.md` - Minus Counter Down Up

### I/O (Block-2 Specific)
- `read_h.md` - Read channel
- `write_h.md` - Write channel
- `inotld_h.md` - I/O Not Load
- `inotrd_h.md` - I/O Not Read
- `rand_h.md` - Read and Mask
- `wand_h.md` - Write and Mask
- `ror_h.md` - Read or Mask
- `wor_h.md` - Write or Mask
- `rxor_h.md` - Read exclusive or

## Source Material

Primary source: **AGCIS Issue 32** (`ref/moon/agcis_32_blk2_instructions.pdf`)
- 94MB PDF with detailed Block-2 instruction descriptions
- Read in chunks during 2025-12-07 documentation pass

Supporting sources:
- **AGCIS Issue 2** - Block-1 baseline for comparison
- **AGCIS Issue 3** - CPU registers and behavior (applies to both blocks)
- **AEA Programming Reference** - I/O scaling and format details

## Differences from Block-1

Key behavioral differences documented in `ref/block2/differences.md`:
- E-memory handling and restore timing
- Extended addressing (EXT instruction behavior)
- Additional subinstructions for some operations
- Counter control enhancements
- New I/O instructions not present in Block-1

## Usage Notes

When working with Block-2 files:
1. Use inline style for small helpers with explanation
2. Note Block-2-specific behaviors in "Notes" section
3. Cross-reference Block-1 versions for comparison
4. Mark uncertain E-memory/EXT behaviors with TODO:VERIFY
5. Update audit blocks with specific AGCIS Issue 32 page refs
6. Follow commit convention: `[AI]` prefix with ISO timestamp

## Related Documentation

- **Block-1 Instructions:** `ref/block1/instr/` - Compare for differences
- **CPU Documentation:** `ref/cpu/` - Shared registers and adder behavior
- **Canonical Definitions:** `ref/definitions/` - Instruction types, STD2, EXTEND
- **Differences Summary:** `ref/block2/differences.md`
- **Index:** `ref/block2/index.md` - Complete instruction listing

---

Last updated: 2025-12-08T04:19:26.568Z
