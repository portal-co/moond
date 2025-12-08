# Source PDF Inventory

> Directory: `ref/moon/`

## Overview

This directory contains the original AGC documentation PDFs that serve as authoritative sources for all instruction and CPU documentation in this repository.

## Available PDFs

### AGCIS (AGC Information Series) Documents

**`agcis_2_machine_instructions.pdf`** (13 MB)
- **Title:** AGCIS Issue 2 - Machine Instructions
- **Content:** Complete Block-1 instruction set with timing diagrams, tables, and detailed descriptions
- **Pages:** ~102 pages of instruction documentation
- **Status:** ✅ Processed - Extracted to `ref/block1/instr/`
- **Key Sections:**
  - Pages 15-36: Core instructions (TC, STD2, XCH, NDX, AD, SU)
  - Pages 46-60: Multiply (MP) subinstructions
  - Pages 61-80: Divide (DV) and related operations
  - Pages 86-102: Special instructions (RPT, PINC/MINC, shifts, etc.)

**`agcis_3_central_processor.pdf`** (6.7 MB)
- **Title:** AGCIS Issue 3 - Central Processor
- **Content:** CPU architecture, registers, adder, parity, write amplifiers
- **Pages:** ~25 pages of CPU subsystem documentation
- **Status:** ✅ Processed - Extracted to `ref/cpu/`
- **Key Sections:**
  - Pages 3-11: Register descriptions (A, Q, Z, LP, B, G, S, SQ)
  - Pages 12-20: Adder and arithmetic unit
  - Pages 21-25: Parity block and error detection

**`agcis_32_blk2_instructions.pdf`** (94 MB)
- **Title:** AGCIS Issue 32 - Block-2 Instructions
- **Content:** Block-2 enhanced instruction set with extended addressing and I/O
- **Pages:** Very large PDF (94 MB), read in chunks
- **Status:** ✅ Processed - Extracted to `ref/block2/`
- **Processing:** Read during 2025-12-07 documentation pass, 41 instruction files created
- **Special:** Contains Block-2-specific enhancements, E-type instructions, additional I/O

### External Memos and References

**`agc4_memo9_rev_june1967.pdf`** (6.7 MB)
- **Title:** AGC4 Memo 9 (Revision June 1967)
- **Content:** External memo with complementary notes and examples
- **Status:** ⏸️ Not yet processed
- **Priority:** Medium (after Block-2 and primary documentation complete)
- **Note:** May contain edge-case clarifications and usage examples

**`AEAProgrammingReference.pdf`** (14 MB)
- **Title:** AEA (Abort Electronics Assembly) Programming Reference
- **Content:** PGNS scaler widths, downlink word formats, I/O channel details
- **Status:** ⏸️ Partially processed - Notes at `ref/aea/README.md`
- **Key Sections:**
  - Pages 15-18: Scaler/register widths and downlink formats
- **Priority:** Low (complementary reference for I/O and scaling details)

## File Count

This directory contains **5 PDF files** (3 fully processed, 2 partially/unprocessed).

## Processing Status Summary

| PDF | Size | Status | Output Location | Priority |
|-----|------|--------|-----------------|----------|
| agcis_2_machine_instructions.pdf | 13 MB | ✅ Complete | ref/block1/instr/ | Done |
| agcis_3_central_processor.pdf | 6.7 MB | ✅ Complete | ref/cpu/ | Done |
| agcis_32_blk2_instructions.pdf | 94 MB | ✅ Complete | ref/block2/ | Done |
| agc4_memo9_rev_june1967.pdf | 6.7 MB | ⏸️ Pending | N/A | Medium |
| AEAProgrammingReference.pdf | 14 MB | ⏸️ Partial | ref/aea/ | Low |

## Lost Media Notice

> **IMPORTANT:** Some AGC documentation is known to be lost media. References to files like `agcis_4` or other materials not present in this directory should trigger a human consultation. Do not assume missing files exist elsewhere.

See the LOST MEDIA NOTICE in `ref/important_goals.md` and `ref/CONVERSATION_SUMMARY.md`.

## Usage Guidelines

### Citing PDFs in Documentation

When referencing these PDFs in instruction documentation:

**Format:**
```markdown
Source: `agcis_2_machine_instructions.pdf` — pages X–Y (figs. A-B; tables C-D).
```

**Examples:**
```markdown
Source: `agcis_2_machine_instructions.pdf` — pages 36-45 (figs. 2-12..2-16; table 2-3).
Source: `agcis_3_central_processor.pdf` — pages 6-12 (table 3-1).
```

### Reading Large PDFs

For large PDFs (especially `agcis_32_blk2_instructions.pdf` at 94 MB):
- **DO NOT** read entire PDF in one go
- Read specific pages or small page ranges (10-20 pages max)
- Use targeted reads for specific instructions
- Reference commit history for already-processed content

### PDF Processing Convention

When extracting from PDFs:
1. Note the PDF name and specific page ranges
2. Create one markdown file per instruction or major topic
3. Add source citation at top of each file
4. Add audit block noting which PDF pages were used
5. Commit with `[AI]` prefix and ISO timestamp

## OCR and Quality Notes

### AGCIS Issue 2 & 3
- Generally good OCR quality
- Most text readable and extractable
- Some figures may be low quality
- Tables generally clear

### AGCIS Issue 32 (Block-2)
- Very large file (94 MB)
- OCR quality varies by section
- Some sections may be scanned images
- Read carefully and mark ambiguities with TODO:VERIFY

### Memos and References
- agc4_memo9: External memo, quality unknown until processed
- AEA Reference: Specialized format, some technical diagrams

## Audit and TODO:VERIFY

When PDF content is ambiguous or OCR-unclear:
1. Mark with `TODO:VERIFY` in extracted documentation
2. Add rationale: "OCR unreadable", "figure unclear", "inferred from context"
3. Log in `ref/TODO_AUDIT.md` for centralized tracking
4. Attempt cross-referencing with other PDFs if available

## Related Documentation

- **Block-1 Instructions:** `ref/block1/instr/` - Extracted from AGCIS Issue 2
- **Block-2 Instructions:** `ref/block2/` - Extracted from AGCIS Issue 32
- **CPU Documentation:** `ref/cpu/` - Extracted from AGCIS Issue 3
- **AEA Notes:** `ref/aea/` - Partial extraction from AEA Programming Reference
- **Source Overviews:** `ref/sources/` - Indexes and overviews of AGCIS Issue 2
- **Audit Tracking:** `ref/TODO_AUDIT.md` - TODO:VERIFY markers and PDF verification status

## Processing History

- **2025-12-07:** Initial AGCIS Issue 2 and Issue 3 processing
- **2025-12-07:** AGCIS Issue 32 (Block-2) processing, 41 files created
- **2025-12-07:** Partial AEA reference extraction (pages 15-18)
- **2025-12-07:** Audit resolution pass with PDF page citations
- **Future:** agc4_memo9 processing (pending, medium priority)

---

Last updated: 2025-12-08T04:19:26.568Z
