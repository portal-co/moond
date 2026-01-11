# Important Goals

> LOST MEDIA NOTICE (2025-12-08T03:55:20.709Z): Some referenced sources (e.g., `agc4_memo9_rev_june1967.pdf`) are not present in the repository and appear to be lost media. Future agents should ask a human when they see these references; do not assume the files exist.

- [x] Prioritize Block-2: read/process ref/moon/agcis_32_blk2_instructions.pdf and produce one Markdown file per instruction under `ref/block2/`.
- [ ] Process complementary sources: `agc4_memo9_rev_june1967.pdf` and `AEAProgrammingReference.pdf`.
- [ ] Finish remaining per-instruction docs (Block-1 & Block-2) and refine pseudocode for edge-cases.
- [x] Unify registers & word-size documentation: make `ref/cpu/registers.md` authoritative.
- [ ] Ensure parity between Block-1 and Block-2 pseudocode styles.
- [ ] Document and mark ambiguous/uncertain behaviors as `TODO:VERIFY` with a short rationale.
- [x] Consolidate all `TODO:VERIFY` items into prioritized lists in `ref/TODO_AUDIT.md`.
- [x] Add an Audit entry inside each `TODO` file pointing to `ref/TODO_AUDIT.md`.
- [x] Add explicit citations (PDF page refs or external memos) for substantial references.
- [x] Create `ref/block2/differences.md` summarizing behavioral divergences vs Block-1.
- [x] Maintain searchable TODO markers and ensure auditability.
- [x] Track progress with commits and consistent messages ([AI] prefix).

## Status Summary (2026-01-10)

**Completed:**
- Block-2 documentation structure (41 files).
- Canonical register and type definitions (ref/cpu/registers.md, ref/definitions/).
- Audit process (TODO_AUDIT.md).
- Citation system (AGCIS Issue 2/3 and AEA page references).
- Block-2 differences.md.
- Directory cleanup (Phase 1 & 2 complete).
- Modernization templates (Phase 3 in progress).

**In Progress:**
- Systematic modernization of instruction files.
- Processing remaining lost/external sources.
- Resolving TODO:VERIFY markers.
