**Conversation Summary — AGCIS Documentation Project**

- **Scope:** Read AGCIS PDFs in `ref/moon`, extract instruction & CPU descriptions, modernize into one Markdown file per instruction (under `ref/instr/`) and place CPU/subsystem docs under `ref/cpu/`. Use C-like pseudocode for micro-ops, use `0o` octal prefixes, and `(u)int15_t` / `int16_t`-style types to reflect AGC word semantics.

**What I’ve done so far**
- Located AGCIS sources in `ref/moon`:
  - `AEAProgrammingReference.pdf`
  - `agc4_memo9_rev_june1967.pdf` (present, but external memo)
  - `agcis_2_machine_instructions.pdf`
  - `agcis_32_blk2_instructions.pdf`
  - `agcis_3_central_processor.pdf`
- Processed `agcis_2_machine_instructions.pdf` (pp.1..102 in chunks):
  - Created per-instruction Markdown files (examples):
    - `ref/instr/ccs_k.md`, `su_k.md`, `mp_k.md`, `dv_k.md`, `tc_k.md`, `xch_k.md`, `csk_k.md`, `ts_k.md`, `msk_k.md`, `ad_k.md`, `ndx_k.md`, and others (RPT, RSM, PINC, MINC, SHINC, SHANC, GO, TCSA, OINC, LINC).
  - Rewrote the combined instructions file `ref/agcis_2_machine_instructions_instructions.md` to point at the per-instruction files and created an index `ref/agcis_2_machine_instructions_index.md` listing expected per-instruction docs.
- Created CPU/subsystem docs from `agcis_3_central_processor.pdf`; files placed under `ref/cpu/`:
  - `central_processor_overview.md`
  - `registers.md`
  - `adder.md`
  - `parity_block.md`
  - `write_amplifiers.md`
  - `bnk.md`
- Organized per-instruction docs under `ref/instr/` and prepared `ref/cpu/` for non-instruction technical docs.

**Pseudocode / Documentation Conventions (applies across files)**
- Style: C-like pseudocode functions that express an instruction's logical micro-op sequence (example: `void CCS_K(uint16_t K) { ... }`).
- Types: use `uint16_t`, `int16_t` for 16-bit words and `(u)int15_t` in prose where 15-bit value fields are important.
- Octal: all octal literals use `0o` prefix (e.g., `0o20`).
- Micro-op primitives: I use compact helpers to represent common AGC behavior:
  - `STMIC_stage()` or inline equivalents to represent the standard memory-inquiry sequence (stage/fetch B/P/S/X/Y, fetch `G` if applicable).
  - `extract_order_code()`, `compute_parity_bit()`, `schedule_PINC()` / `schedule_MINC()` as documentation helpers mirroring AGC control pulses.
- Subinstructions: original AGC uses multi-part subinstructions (e.g., `STD2`, `RPT1`, `RPT3`). For readability and emulation, I inline those into single atomic routines that preserve functional effects.

**Files I created (high-level)**
- Instruction docs (excerpt): `ref/instr/ccs_k.md`, `su_k.md`, `mp_k.md`, `dv_k.md`, `tc_k.md`, `xch_k.md`, `csk_k.md`, `ts_k.md`, `msk_k.md`, `ad_k.md`, `ndx_k.md`, `rpt.md`, `rsm.md`, `pinc.md`, `minc.md`, `shinc.md`, `shanc.md`, `go.md`, `tcsa.md`, `oinc.md`, `linc.md`.
- CPU docs: `ref/cpu/central_processor_overview.md`, `registers.md`, `adder.md`, `parity_block.md`, `write_amplifiers.md`, `bnk.md`.
- Index & redirects: `ref/agcis_2_machine_instructions_index.md`, updated `ref/agcis_2_machine_instructions_instructions.md` to point to `ref/instr/`.

**Todo / Goals (current)**
- Immediate next task (priority): Read and process `agcis_32_blk2_instructions.pdf` and produce one-file-per-instruction docs for Block-2 instructions. Place Block-2-specific docs — and any differences vs Block-1 — under `ref/block2/`.
- Then: process `agc4_memo9_rev_june1967.pdf` and `AEAProgrammingReference.pdf` as they contain complementary notes and examples.
- Ongoing: finish any remaining instruction files discovered during reading, refine pseudocode for edge-cases (divide-by-zero, exact pinned timing), and add cross-file links & citations.

**Block-2 (special handling)**
- `agcis_32_blk2_instructions.pdf` documents Block-2 (similar but not identical) architecture. I will:
  - Create `ref/block2/` and place Block-2 instruction docs there.
  - Note differences inline in each Block-2 file and, when useful, add a short `ref/block2/differences.md` summarizing behavioral divergences vs the Block-1 (AGCIS Issue 2) semantics.

**Missing / OCR / Lost Media notes**
- I confirmed the PDFs currently in `ref/moon` (see list above). Some AGCIS 4 material is known to be lost media (you flagged `agcis_4` as missing), and some files may not be OCR'd or may be low-quality. If I encounter in-text references to PDFs or sections that are not present in `ref/moon`, I will stop and notify you immediately.
- Current status: `agcis_2_machine_instructions.pdf` and `agcis_3_central_processor.pdf` were OCR'd and successfully read. `agcis_32_blk2_instructions.pdf` is present and will be processed next. `agc4_memo9_rev_june1967.pdf` is present but may be a memo with different coverage; treat as secondary.

**How I track progress**
- I maintain a tracked TODO list in the workspace (used to coordinate multi-step work). I update it after each significant milestone.

**Next actions (I will start when you confirm)**
- Read `agcis_32_blk2_instructions.pdf` in page-chunks, extract instruction descriptions, and create per-instruction Markdown files under `ref/block2/` (keeping the same C-like pseudocode conventions and `0o` octal style). I'll highlight differences to Block-1 in each file and add a `ref/block2/differences.md` summarizing any architecture-level changes.

If you want me to proceed now, reply with “Proceed” and I will begin reading `agcis_32_blk2_instructions.pdf`. If you prefer adjustments to naming, pseudocode style, or file layout first, tell me which change and I will apply it before continuing.

---

**In-progress — Block-2 work (AI agent):**
- Status: initial Block-2 instruction docs created under `ref/block2/` (index + 10 files). Continue work by expanding these per-instruction Markdown files; do not re-read the entire PDF unless explicitly requested.
- Progress is tracked by commits; prefer reading recent commit messages (AI commits start with "[AI]") for authoritative, agentic decisions and details.
- Next steps: expand each Block-2 instruction file with full prose, subinstruction mapping, and Block-2 differences; produce commits per batch with "[AI]"-prefixed messages.
- Last update: 2025-12-07T07:34:46.853Z

---

[Instruction type documented: `ref/Instruction.md`]

**Recent requirement changes (2025-12-07T07:51:09Z):**
- C-like pseudocode required in all Block-2 and instruction docs (use uint16_t/int16_t and the Instruction typedef).
- Instruction type documented at ref/Instruction.md; STD2 and EXTEND helpers documented at ref/STD2.md and ref/EXTEND.md — per-instruction docs should reference these helpers.
- Commit messages for AI-generated edits must start with "[AI]" and include an ISO timestamp; do NOT modify the repo git user config.
- Avoid re-reading the entire Block-2 PDF; process in targeted page-chunks and prefer reading recent commit messages for agentic context.

*File created: `ref/CONVERSATION_SUMMARY.md`*

