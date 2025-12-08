REVIEW INSTRUCTIONS — Pseudocode and Parity

Generated: 2025-12-07T08:56:46.023Z

Purpose
- Provide a concise checklist and style guide for reviewing the C-like pseudocode files created in ref/block2/pseudocode/ (and later ref/block1/pseudocode/).

Review checklist
1) Header/metadata: each pseudocode file must include a one-line generated-by header, source reference(s) (AGCIS Issue 2/3, AEA pages), and a timestamp.
2) Function signature: use a clear C-like function name (snake_case or CamelCase consistent across files); include operand types where known.
3) Steps: follow the canonical structure: STMIC (if memory access), load operands, core operation (use helper functions), overflow/edge-case handling, memory write-back, SQG/STD2 sequencing.
4) Helpers & naming: prefer helper functions add_with_flags, signed_multiply_14x14, agc_divide_approach3, test_parity, MEM.read/MEM.write, and SQG operations. Document helper semantics in file comments.
5) TODO markers: keep TODO:VERIFY for hardware/timing/edge-case items; use TODO:REVIEW for items needing human review. Mark unknown results explicitly.
6) Inline small functions: in Block-2 prefer inlining small helper bodies with a comment referencing the helper name; add an annotation on the inlinee explaining why it was inlined.
7) Parity & signbit: document bit-15/bit-16 movement when transferring between registers and memory; reference AGCIS Issue 3 pages 3–11.
8) Commit messages: use messages starting with "[AI]" for automated commits; include brief description and timestamp.
9) Pending list: pending conversions are tracked in ref/TEMP_INSTR_CHANGES.md. Do NOT remove entries until a human reviewer confirms the pseudocode is final; an AI may append an "expanded" block but must not delete the pending entry without explicit human approval.

How to mark a file as reviewed
- After checking file matches style and resolving TODO:VERIFY items with authoritative citations, add a short "Reviewed" block in the file with reviewer name/date and then remove the entry from ref/TEMP_INSTR_CHANGES.md.

Contact points
- For ambiguous AGC behavior (EXT/STD2 timing, E-memory restore race conditions, SCALER formats), attach citations from AGC memos or NASA/Raytheon notes when available; otherwise mark TODO:VERIFY and escalate.

