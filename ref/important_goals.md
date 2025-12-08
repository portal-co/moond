# Important Goals

> LOST MEDIA NOTICE (2025-12-08T03:55:20.709Z): Some referenced sources (e.g., `agc4_memo9_rev_june1967.pdf`) are not present in the repository and appear to be lost media. Future agents should ask a human when they see these references; do not assume the files exist.


_Extracted from ref/CONVERSATION_SUMMARY.md and ref/TODO_AUDIT.md — created: 2025-12-08T03:51:20.428Z_

- [ ] Prioritize Block-2: read/process ref/moon/agcis_32_blk2_instructions.pdf and produce one Markdown file per instruction under `ref/block2/`, noting differences vs Block-1.
- [ ] Process complementary sources: `agc4_memo9_rev_june1967.pdf` and `AEAProgrammingReference.pdf` and extract supporting notes/examples.
- [ ] Finish remaining per-instruction docs (Block-1 & Block-2) and refine pseudocode for edge-cases (divide-by-zero, overflow, EXT timing).
- [ ] Unify registers & word-size documentation: make `ref/cpu/registers.md` (or canonical doc under `ref/definitions/`) authoritative and reference it from all instruction files.
- [ ] Ensure parity between Block-1 and Block-2 pseudocode styles (canonical helpers for Block-1; inline small helpers with "Inline notes" for Block-2).
- [ ] Document and mark ambiguous/uncertain behaviors as `TODO:VERIFY` with a short rationale (e.g., "inferred from training/model", "OCR unreadable").
- [ ] Consolidate all `TODO:VERIFY` items into prioritized lists (e.g., EXT timing, E-memory, SCALER formats) and seek authoritative sources (AGC memos, hardware tests); update `ref/TODO_AUDIT.md` with results.
- [ ] Add an Audit entry inside each `TODO` file pointing to the central summary (`ref/TODO_AUDIT.md`) describing local rationale and status.
- [ ] Add explicit citations (PDF page refs or external memos) for substantial references (SCALER channel widths, bank timing rules, parity tables) when added to docs.
- [ ] Create `ref/block2/differences.md` summarizing behavioral divergences vs Block-1 and link from block2 instruction files.
- [ ] Maintain searchable TODO markers and ensure auditability (keep `TODO:VERIFY` tags concise and searchable).
- [ ] Track progress with commits and consistent messages (project convention: AI-generated commits start with "[AI]" and ISO timestamps).

Notes:
- Priority order: 1) Block-2, 2) Block-2 base / CPU/register behaviors, 3) Block-1 parity, 4) AEA references.
- For ambiguous items, prefer authoritative sourcing; if unavailable, capture unit-test/emulation validation attempts and document outcomes.
