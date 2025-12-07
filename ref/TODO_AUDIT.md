TODO:VERIFY Audit Log — 2025-12-07T08:13:47.632Z

Summary
- This file lists all repository markdown files that contain `TODO:VERIFY` markers discovered during the parity/refinement pass and records the initial audit status.
- PDFs searched for authoritative evidence: ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf (where present in repo). If further authoritative sources are known, add them below.

Audit procedure performed:
- Searched repo markdowns for `TODO:VERIFY` markers (grep over ref/).
- Sampled central PDFs (AEAProgrammingReference.pdf and AGCIS Central Processor) for related terminology and cross-checked obvious scale/format references where present.
- Where no definitive support was found in repo PDFs or the PDFs were ambiguous/OCR'd, items remain flagged `TODO:VERIFY` and are noted below with audit status.

General result
- Many `TODO:VERIFY` markers reference behaviors or details (EXT timing, overflow-bit propagation, SCALER channel widths, E-memory restore timing) that are not unambiguously documented in the repo's PDFs or in the OCR'd text available.
- For each file below the initial audit status is listed. Most entries require further external sourcing (memos, hardware tests, or subject-matter confirmation) and therefore retain their `TODO:VERIFY` markers.

Files with TODO:VERIFY (initial audit)
- ref/block2/das_e.md: TODO:VERIFY (searched PDFs; ambiguous) — Action: retain marker; needs hardware memo or original AGC notes.
- ref/block2/pcdu_c.md: TODO:VERIFY (searched PDFs; ambiguous)
- ref/block2/ca_k.md: TODO:VERIFY (memory bank selection; searched PDFs; not found)
- ref/block2/shanc_c.md: TODO:VERIFY (ambiguous)
- ref/block2/read_h.md: TODO:VERIFY (SCALER channel width alignment; searched AEA doc; AEA shows scale factors but not explicit SCALER channel format)
- ref/block2/fetch_k.md: TODO:VERIFY (ambiguous)
- ref/block2/differences.md: TODO:VERIFY (ambiguous)
- ref/block2/index.md: TODO:VERIFY (ambiguous)
- ref/block2/resume.md: TODO:VERIFY (ambiguous)
- ref/block2/ndx_e.md: TODO:VERIFY (ambiguous)
- ref/block2/pinc_c.md: TODO:VERIFY (ambiguous)
- ref/block2/mcdu_c.md: TODO:VERIFY (ambiguous)
- ref/block2/aug_e.md: TODO:VERIFY (ambiguous)
- ref/block2/tc_k.md: TODO:VERIFY (EXT handling; noted in file)
- ref/block2/msu_e.md: TODO:VERIFY (overflow vs sign-bit for cyclic results)
- ref/block2/shinc_c.md: TODO:VERIFY (ambiguous)
- ref/block2/wor_h.md: TODO:VERIFY (ambiguous)
- ref/block2/minc_c.md: TODO:VERIFY (ambiguous)
- ref/block2/rxor_h.md: TODO:VERIFY (ambiguous)
- ref/block2/ad_k.md: TODO:VERIFY (overflow flags encoding)
- ref/block2/rand_h.md: TODO:VERIFY (ambiguous)
- ref/block2/xch_e.md: TODO:VERIFY (overflow-bit propagation with E-memory)
- ref/block2/mp_k.md: TODO:VERIFY (product overflow encoding timing)
- ref/block2/inotrd_h.md: TODO:VERIFY (ambiguous)
- ref/block2/dim_e.md: TODO:VERIFY (ambiguous)
- ref/block2/dinc_c.md: TODO:VERIFY (ambiguous)
- ref/block2/inotld_h.md: TODO:VERIFY (ambiguous)
- ref/block2/tcf_f.md: TODO:VERIFY (EXT requirement relative to STD2)
- ref/block2/wand_h.md: TODO:VERIFY (ambiguous)
- ref/block2/ads_e.md: TODO:VERIFY (ambiguous)
- ref/block2/ror_h.md: TODO:VERIFY (ambiguous)
- ref/block2/incr_e.md: TODO:VERIFY (ambiguous)
- ref/block2/ror_h.md: TODO:VERIFY (ambiguous)
- ref/block2/ads_e.md: TODO:VERIFY (ambiguous)
- ref/block2/dv_e.md: TODO:VERIFY (ambiguous)
- ref/block2/store_e.md: TODO:VERIFY (ambiguous)
- ref/block2/rupt.md: TODO:VERIFY (ambiguous)
- ref/block2/tcsaj_k.md: TODO:VERIFY (ambiguous)
- ref/block2/go.md: TODO:VERIFY (ambiguous)
- ref/block2/ccs_e.md: TODO:VERIFY (plus/minus-zero encoding; noted in file)
- ref/block2/dv_e.md: TODO:VERIFY (ambiguous)
- ref/block2/su_e.md: TODO:VERIFY (ambiguous)
- ref/block2/write_h.md: TODO:VERIFY (ambiguous)
- ref/block2/rupt.md: TODO:VERIFY (ambiguous)

Notes and next steps
- Action recommendation: consolidate `TODO:VERIFY` items into prioritized lists (e.g., EXT timing, E-memory editing, SCALER formats) and attempt to locate authoritative sources (AGC memos, hardware test logs, or original NASA/Raytheon memos). If these sources are unavailable, consider running unit tests in the emulation against known AGC behavior to empirically validate ambiguous items.
- If desired, the next automated step is: for each `TODO` file, add an Audit entry inside the file pointing to this central summary and describing the local rationale; continue converting ambiguous inferences into `TODO:VERIFY (inferred from training/model)` where applicable.

-- End of initial audit

Audit progress (2025-12-07T08:30:24.624Z):
- Resolved/Tentatively-resolved items have been annotated in individual files that reference AGCIS Issue 2/3 and AEA pages (see appended Audit resolution blocks).
- Remaining TODO:VERIFY entries require external memos/hardware logs or deeper PDF page reads; they remain listed above for future action.

Audit resolution (2025-12-07T08:34:19.588Z):
- Targeted sources reviewed: AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf) pages 15–36, 46–60, 61–80, 86–102; AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf) pages 3–11; AEAProgrammingReference.pdf pages 15–18 where applicable.
- Behavior matching these sources is considered supported and marked resolved in-file when specific; remaining ambiguous details retain TODO:VERIFY and are listed in ref/TODO_AUDIT.md for later authoritative sourcing.

Resolution (2025-12-07T08:37:28.578Z):
- Supported behaviors referenced in this file have been corroborated by targeted readings of AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf; pages ~15–36, 46–60, 61–80, 86–102), AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf; pages 3–11), and AEAProgrammingReference.pdf (ref/moon/AEAProgrammingReference.pdf; pp.15–18) where applicable.
- Status: instruction semantics and register-transfer behaviors supported by these sources are considered resolved here; hardware timing/edge-case details remain TODO:VERIFY and are tracked centrally in ref/TODO_AUDIT.md for later authoritative sourcing.
