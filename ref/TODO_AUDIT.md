TODO:VERIFY Audit Log

## Overview
This file tracks all `TODO:VERIFY` markers discovered during the project. It serves as a central registry for items requiring further authoritative sourcing (memos, hardware tests, or subject-matter confirmation).

## Audit Status (Last Update: 2025-12-07T08:37:28.578Z)
- **Resolved/Corroborated:** Instruction semantics and register-transfer behaviors matching AGCIS Issue 2 (pp. 15–36, 46–60, 61–80, 86–102), AGCIS Issue 3 (pp. 3–11), and AEA Programming Reference (pp. 15–18) have been marked as resolved in individual files.
- **Remaining:** Hardware timing, edge-cases (EXT timing, overflow-bit propagation, SCALER channel widths, E-memory restore timing), and ambiguous OCR results remain flagged for further action.

## Files with TODO:VERIFY (Initial Audit)
- ref/block2/das_e.md: Ambiguous (needs hardware memo)
- ref/block2/pcdu_c.md: Ambiguous
- ref/block2/ca_k.md: Memory bank selection (not found in current PDFs)
- ref/block2/shanc_c.md: Ambiguous
- ref/block2/read_h.md: SCALER channel width alignment (AEA shows scale factors but not explicit format)
- ref/block2/fetch_k.md: Ambiguous
- ref/block2/differences.md: Ambiguous
- ref/block2/index.md: Ambiguous
- ref/block2/resume.md: Ambiguous
- ref/block2/ndx_e.md: Ambiguous
- ref/block2/pinc_c.md: Ambiguous
- ref/block2/mcdu_c.md: Ambiguous
- ref/block2/aug_e.md: Ambiguous
- ref/block2/tc_k.md: EXT handling
- ref/block2/msu_e.md: overflow vs sign-bit for cyclic results
- ref/block2/shinc_c.md: Ambiguous
- ref/block2/wor_h.md: Ambiguous
- ref/block2/minc_c.md: Ambiguous
- ref/block2/rxor_h.md: Ambiguous
- ref/block2/ad_k.md: overflow flags encoding
- ref/block2/rand_h.md: Ambiguous
- ref/block2/xch_e.md: overflow-bit propagation with E-memory
- ref/block2/mp_k.md: product overflow encoding timing
- ref/block2/inotrd_h.md: Ambiguous
- ref/block2/dim_e.md: Ambiguous
- ref/block2/dinc_c.md: Ambiguous
- ref/block2/inotld_h.md: Ambiguous
- ref/block2/tcf_f.md: EXT requirement relative to STD2
- ref/block2/wand_h.md: Ambiguous
- ref/block2/ads_e.md: Ambiguous
- ref/block2/ror_h.md: Ambiguous
- ref/block2/incr_e.md: Ambiguous
- ref/block2/dv_e.md: Ambiguous
- ref/block2/store_e.md: Ambiguous
- ref/block2/rupt.md: Ambiguous
- ref/block2/tcsaj_k.md: Ambiguous
- ref/block2/go.md: Ambiguous
- ref/block2/ccs_e.md: plus/minus-zero encoding
- ref/block2/su_e.md: Ambiguous
- ref/block2/write_h.md: Ambiguous

## Next Steps
- Consolidate items into prioritized categories: EXT timing, E-memory, SCALER formats.
- Locate original NASA/Raytheon memos or run unit tests in emulation to empirically validate ambiguous behaviors.
