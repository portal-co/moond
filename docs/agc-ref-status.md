# AGC Reference Documentation Status

Tracks the state of the `ref/` directory: what has been processed, what remains, and the modernization backlog.

## Project Scope

The `ref/` directory contains documentation extracted from AGC source PDFs, organized into per-instruction Markdown files. Conventions use modern C-like pseudocode, `0o` octal prefixes, and `(u)int15_t` / `int16_t` types. See `ref/CONVENTIONS.md` for full style guide.

Source PDFs (in `ref/moon/`):
- `agcis_2_machine_instructions.pdf` — Block-1 instruction set (AGCIS Issue 2)
- `agcis_3_central_processor.pdf` — Block-1/2 CPU internals (AGCIS Issue 3)
- `agcis_32_blk2_instructions.pdf` — Block-2 instruction set (AGCIS Issue 32)
- `AEAProgrammingReference.pdf` — AEA Programming Reference
- `agc4_memo9_rev_june1967.pdf` — **LOST MEDIA**: not present in repo; ask a human if you need it

## What is Complete

- Block-2 instruction documentation — 41 files in `ref/block2/`
- Block-1 instruction documentation — 21 files in `ref/block1/instr/`
- Canonical CPU register/word-size docs — `ref/cpu/registers.md` authoritative
- Canonical type and helper definitions — `ref/definitions/`
- Differences between Block-1 and Block-2 — `ref/block2/differences.md`
- Audit system for `TODO:VERIFY` markers — `ref/TODO_AUDIT.md`
- Citation system (AGCIS Issue 2/3/32, AEA page refs)
- Directory READMEs for all major subdirectories
- Conventions document — `ref/CONVENTIONS.md`
- `ref/` structural cleanup (duplicates archived, pseudocode stubs archived)

## Remaining Work

### Modernization Backlog (~50+ instruction files)

Many instruction files still use the old hardware-pulse style (e.g., `STMIC_stage()`, `STD2()` calls, pulse names like `RZ`, `WS`, `WG`). The target modern style is documented in `ref/CONVENTIONS.md`. Ten template files were created as reference:

**Block-1 templates:** `ad_k.md`, `ccs_k.md`, `tc_k.md`  
**Block-2 templates:** `ad_k.md`, `ccs_e.md`, `read_h.md`, `aug_e.md`, `incr_e.md`, `dim_e.md`, `ca_k.md`

Priority for modernization:
1. Files with missing PDF citations (15+ Block-2 files including `ads_e`, `bzf_f`, `dv_e`, and others)
2. Files with hardware pulse names — search `STMIC_stage\|STD2()\|RZ\|WG`
3. Systematic application after templates established

### TODO:VERIFY Resolution (~214 markers)

All markers have rationale (none are bare). Tracked in `ref/TODO_AUDIT.md`. Remaining ambiguities:
- Hardware timing (EXT timing, overflow-bit propagation, SCALER channel widths, E-memory restore timing)
- Items dependent on `agc4_memo9_rev_june1967.pdf` (lost media)
- OCR-unclear results awaiting hardware test confirmation

### Content Validation (not started)

- Audit instruction files < 500 bytes for stub content
- Standardize file headers (ensure all have: source citation, summary, pseudocode, audit block)
- Cross-check `TODO_AUDIT.md` count vs actual `grep -r "TODO:VERIFY" ref/` count

### Source Processing (not started)

- Process `AEAProgrammingReference.pdf` beyond the I/O channel sections already referenced
- Process `agc4_memo9_rev_june1967.pdf` — if/when recovered
- Add `ref/sources/` overview files for AGCIS Issue 3 and Issue 32

## Validation Commands

```bash
# Count TODO:VERIFY markers
grep -r "TODO:VERIFY" ref/ | wc -l

# Files still using old pseudocode style
grep -rl "STMIC_stage\|STD2()\|STD1()" ref/block1 ref/block2

# Files missing source citation
grep -rL "AGCIS\|agcis\|AEA" ref/block2/*.md ref/block1/instr/*.md

# Check file sizes for potential stubs
find ref/block1 ref/block2 -name "*.md" -size -500c
```
