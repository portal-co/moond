**Conversation Summary — AGCIS Documentation Project**

- **Scope:** Extract instruction & CPU descriptions from AGCIS PDFs, modernize into Markdown files.
- **Conventions:** Modern C-like pseudocode, `0o` octal prefixes, `(u)int15_t` / `int16_t` types.
- **Status:** Phase 1 (Cleanup) and Phase 2 (READMEs/Conventions) complete. Phase 3 (Modernization/Validation) in progress.

**Key Achievements**
- Processed `agcis_2_machine_instructions.pdf` (Block-1) and `agcis_32_blk2_instructions.pdf` (Block-2).
- Created canonical CPU docs in `ref/cpu/` and definitions in `ref/definitions/`.
- Established audit system for `TODO:VERIFY` markers in `ref/TODO_AUDIT.md`.
- Modernized core instruction files using a unified pseudocode style.

**In-progress**
- Systematic modernization of all instruction files (approx. 60 remaining).
- Resolution of 500+ `TODO:VERIFY` instances.
- Processing `agc4_memo9_rev_june1967.pdf` (if available).

*Last update: 2026-01-10*
