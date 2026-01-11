# AGCIS Issue 32 — Block-2 instruction docs

This directory contains one Markdown file per Block-2 instruction, modernized into C-like pseudocode and prose describing behavioral differences vs Block-1 (AGCIS Issue 2).

Conventions
- Pseudocode uses C-like functions (e.g., `void TC_K(uint16_t K)`), `uint16_t`/`int16_t` for 16-bit words, and `0o` octal prefixes for octal literals.
- Subinstructions (STD2, etc.) are inlined with reference comments pointing to canonical definitions.
- Memory operations use direct `memory[address]` access rather than helper functions.
- Modern terminology: "branch", "fetch", "store", "decode" instead of hardware pulse names.

Files created (initial set)
- tc_k.md
- tcf_f.md
- ccs_e.md
- bzf_f.md
- ca_k.md
- xch_e.md
- ndx_e.md
- mp_k.md
- dv_e.md
- msu_e.md

Generated: 2025-12-07T07:26:15.064Z

Inline notes
- Block-2 docs inline small STMIC stages and micro-ops to preserve fused subinstruction timing; canonical helpers live in ref/definitions and ref/cpu/registers.md.

Edge cases / TODOs
- TODO:VERIFY ambiguous behaviors (overflow bits, EXT timing, E-memory restore timing). See ref/CONVERSATION_SUMMARY.md for tracking.

Audit
- Scanned repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative support; if evidence exists it is noted here. Initial audit: authoritative support not found in repo PDFs or ambiguous/OCR-unclear, so this file retains `TODO:VERIFY` and is provisionally marked as "inferred from training/model" when applicable.
- Action: retain `TODO:VERIFY` marker and consult ref/TODO_AUDIT.md for central tracking. If additional AGC memos or hardware logs are available, add citations below or update this Audit block.

Audit resolution (2025-12-07T08:33:47.148Z):
- Reviewed AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf) targeted pages and AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf) pages 3–11; corroborating instruction flow (STD2), NDX/EXTEND, PINC/MINC, SHINC/SHANC, MP/DV sequences, and register transfer rules.
- Where specific behavior (shift-and-add semantics, overflow counter operations, end-around carry prevention, UPRUPT signaling) is described in this file, it is supported by the cited PDFs and may be considered resolved; remaining nuanced timing/edge-case items retain TODO:VERIFY pending hardware memos.
- See ref/TODO_AUDIT.md for centralized tracking of unresolved items.

Resolution (2025-12-07T08:37:28.578Z):
- Supported behaviors referenced in this file have been corroborated by targeted readings of AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf; pages ~15–36, 46–60, 61–80, 86–102), AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf; pages 3–11), and AEAProgrammingReference.pdf (ref/moon/AEAProgrammingReference.pdf; pp.15–18) where applicable.
- Status: instruction semantics and register-transfer behaviors supported by these sources are considered resolved here; hardware timing/edge-case details remain TODO:VERIFY and are tracked centrally in ref/TODO_AUDIT.md for later authoritative sourcing.
