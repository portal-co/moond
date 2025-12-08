# Source Documentation & Overviews

> Directory: `ref/sources/`

## Overview

This directory contains overview documents and indexes extracted from the source PDFs, primarily from AGCIS Issue 2 (Machine Instructions). These files provide high-level views and navigation aids for the instruction set.

## Files

**`agcis_2_machine_instructions.md`** - AGCIS Issue 2 Overview
- High-level summary of AGCIS Issue 2 content
- Machine instruction organization and structure
- Reference for understanding the original PDF layout

**`agcis_2_machine_instructions_index.md`** - Instruction Index
- Complete index of Block-1 instructions from AGCIS Issue 2
- Lists expected per-instruction documentation files
- Cross-references to `ref/block1/instr/` files

**`agcis_2_machine_instructions_instructions.md`** - Instruction Redirects
- Points to individual instruction files in `ref/block1/instr/`
- Legacy file from initial extraction phase
- Maintained for reference but no longer primary navigation

## File Count

This directory contains **3 source overview/index files**.

## Purpose

These files serve as:
1. **Historical record** - Document what was extracted from AGCIS Issue 2
2. **Navigation aids** - Help locate specific instruction documentation
3. **Coverage tracking** - Verify all instructions from PDFs are documented
4. **Context** - Provide overview of source material organization

## Source Material

These files are based on:
- **AGCIS Issue 2** (`ref/moon/agcis_2_machine_instructions.pdf`)
  - Pages 1-102: Complete Block-1 instruction set
  - Figures, tables, timing diagrams
  - Extracted during initial documentation pass (2025-12-07)

## Relationship to Other Documentation

### Describes
- Content and structure of `ref/moon/agcis_2_machine_instructions.pdf`
- Organization of Block-1 instruction documentation

### Points To
- Individual instruction files in `ref/block1/instr/`
- CPU subsystem docs in `ref/cpu/`

### Complemented By
- `ref/block2/index.md` - Similar index for Block-2 instructions
- `ref/important_goals.md` - Tracks which source PDFs have been processed

## Status

### Completeness
- AGCIS Issue 2 overview and index complete
- All Block-1 instructions from Issue 2 have been extracted
- Individual instruction files created in `ref/block1/instr/`

### Limitations
- Indexes are static snapshots from extraction date
- May not reflect post-extraction refinements to instruction files
- Limited coverage of AGCIS Issue 3 and Issue 32 (those have separate docs)

## Future Additions

As other source PDFs are processed, add similar overview files:
- `agcis_3_overview.md` - Central Processor (already documented in ref/cpu/)
- `agcis_32_overview.md` - Block-2 instructions (index at ref/block2/index.md)
- `agc4_memo9_overview.md` - If/when memo is processed
- `aea_reference_overview.md` - AEA programming reference (notes at ref/aea/)

## Usage Notes

These files are primarily for:
- **Reference** - Understanding source PDF content
- **Navigation** - Finding instruction documentation
- **Auditing** - Verifying extraction completeness

They are **not** authoritative instruction documentation. For actual instruction behavior, see:
- `ref/block1/instr/<instruction>.md` - Individual Block-1 instructions
- `ref/block2/<instruction>.md` - Individual Block-2 instructions

## Related Documentation

- **Block-1 Instructions:** `ref/block1/instr/` - Actual instruction documentation
- **Block-2 Instructions:** `ref/block2/` - Block-2 instruction documentation
- **Source PDFs:** `ref/moon/` - Original PDF files
- **PDF Inventory:** `ref/moon/README.md` - List of available PDFs (to be created)

---

Last updated: 2025-12-08T04:19:26.568Z
