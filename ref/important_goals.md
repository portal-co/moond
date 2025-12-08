# Important Goals

> LOST MEDIA NOTICE (2025-12-08T03:55:20.709Z): Some referenced sources (e.g., `agc4_memo9_rev_june1967.pdf`) are not present in the repository and appear to be lost media. Future agents should ask a human when they see these references; do not assume the files exist.


_Extracted from ref/CONVERSATION_SUMMARY.md and ref/TODO_AUDIT.md — created: 2025-12-08T03:51:20.428Z_

- [x] Prioritize Block-2: read/process ref/moon/agcis_32_blk2_instructions.pdf and produce one Markdown file per instruction under `ref/block2/`, noting differences vs Block-1. (Completed: 41 Block-2 instruction files created; see 2025-12-07 commits)
- [ ] Process complementary sources: `agc4_memo9_rev_june1967.pdf` and `AEAProgrammingReference.pdf` and extract supporting notes/examples. (Partially done: AEA notes at ref/aea/README.md; memo not yet processed)
- [ ] Finish remaining per-instruction docs (Block-1 & Block-2) and refine pseudocode for edge-cases (divide-by-zero, overflow, EXT timing).
- [x] Unify registers & word-size documentation: make `ref/cpu/registers.md` (or canonical doc under `ref/definitions/`) authoritative and reference it from all instruction files. (Completed: ref/cpu/registers.md contains canonical types; ref/definitions/ has Instruction.md, STD2.md, EXTEND.md)
- [ ] Ensure parity between Block-1 and Block-2 pseudocode styles (canonical helpers for Block-1; inline small helpers with "Inline notes" for Block-2).
- [ ] Document and mark ambiguous/uncertain behaviors as `TODO:VERIFY` with a short rationale (e.g., "inferred from training/model", "OCR unreadable").
- [ ] Consolidate all `TODO:VERIFY` items into prioritized lists (e.g., EXT timing, E-memory, SCALER formats) and seek authoritative sources (AGC memos, hardware tests); update `ref/TODO_AUDIT.md` with results.
- [x] Add an Audit entry inside each `TODO` file pointing to the central summary (`ref/TODO_AUDIT.md`) describing local rationale and status. (Completed: Audit blocks added to Block-2 files during 2025-12-07 resolution pass)
- [x] Add explicit citations (PDF page refs or external memos) for substantial references (SCALER channel widths, bank timing rules, parity tables) when added to docs. (Completed: citations added during audit resolution; see TODO_AUDIT.md)
- [x] Create `ref/block2/differences.md` summarizing behavioral divergences vs Block-1 and link from block2 instruction files. (Completed: exists with placeholder content and audit notes)
- [x] Maintain searchable TODO markers and ensure auditability (keep `TODO:VERIFY` tags concise and searchable). (Completed: 76 TODO:VERIFY markers tracked across 104 markdown files)
- [x] Track progress with commits and consistent messages (project convention: AI-generated commits start with "[AI]" and ISO timestamps). (Completed: convention established and followed; see git log)

Notes:
- Priority order: 1) Block-2, 2) Block-2 base / CPU/register behaviors, 3) Block-1 parity, 4) AEA references.
- For ambiguous items, prefer authoritative sourcing; if unavailable, capture unit-test/emulation validation attempts and document outcomes.

## Status Summary (2025-12-08T04:04:22.664Z)

**Completed:**
- Block-2 documentation structure created with 41 instruction files
- Canonical register and type definitions established (ref/cpu/registers.md, ref/definitions/)
- Audit process established with TODO_AUDIT.md tracking 76 TODO:VERIFY markers
- Citation system implemented with AGCIS Issue 2/3 and AEA page references
- Git commit convention established ([AI] prefix with ISO timestamps)
- Block-2 differences.md created

**In Progress / Not Yet Complete:**
- Finish remaining per-instruction docs (Block-1 & Block-2) - many files are stubs or incomplete
- Refine pseudocode for edge-cases (divide-by-zero, overflow, EXT timing) - marked with TODO:VERIFY
- Ensure parity between Block-1 and Block-2 pseudocode styles - partial parity achieved
- Document and mark ambiguous/uncertain behaviors - 76 TODO:VERIFY markers remain
- Consolidate all TODO:VERIFY items into prioritized lists - initial audit done but resolution incomplete
- Process agc4_memo9_rev_june1967.pdf - not started
- Process AEAProgrammingReference.pdf - partial (notes at ref/aea/README.md)

**File Counts:**
- Block-2 instruction files: 41 (plus 21 pseudocode stubs in ref/block2/pseudocode/)
- Block-1 instruction files: 21
- TODO:VERIFY markers: 76 across 104 markdown files
- PDFs available: 5 (all in ref/moon/)
