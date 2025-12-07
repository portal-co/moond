# AGCIS Issue 32 — Block-2 instruction docs

This directory will contain one Markdown file per Block-2 instruction, modernized into C-like pseudocode and prose describing behavioral differences vs Block-1 (AGCIS Issue 2).

Conventions
- Pseudocode uses C-like functions (e.g., `void TC_K(uint16_t K)`), `uint16_t`/`int16_t` for 16-bit words, and `0o` octal prefixes for octal literals.
- Subinstructions (STD2, RUPT0, etc.) are presented as single atomic routines in the pseudocode.
- Memory read/write helper: `STMIC_stage()` indicates the standard memory-inquiry/fetch sequence (fetch B/S/X/Y and G where applicable).

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
