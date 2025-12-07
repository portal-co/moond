**AGCIS Issue 2 — Instruction Index**

- **Scope:** Compact index of machine instructions converted from `agcis_2_machine_instructions.pdf` into one-file-per-instruction docs under `ref/instr/`.
- **Conventions:** Octal numbers use `0o` prefix. Pseudocode uses C-like signatures and `(u)int15_t`/`int16_t` types for AGC word semantics.

- **TC K:** `ref/instr/tc_k.md` — Transfer control to `K`, save return address in `Q`. (AGCIS pp.18–19)
- **NDX K:** `ref/instr/ndx_k.md` — Index the *next* instruction by adding `[K]` to the instruction at `z+1`. (AGCIS pp.33–36)
- **CCS K:** `ref/instr/ccs_k.md` — Count/Compare/Skip: branch based on sign/zero of `[K]`; sets `A` accordingly. (AGCIS pp.36–40)
- **SU K:** `ref/instr/su_k.md` — Subtract `[K]` from `A`; schedules PINC/MINC on overflow/underflow. (AGCIS pp.28–35)
- **AD K:** `ref/instr/ad_k.md` — Add `[K]` into `A`; update overflow counter on OF/UF. (AGCIS pp.31–33)
- **XCH K:** `ref/instr/xch_k.md` — Exchange `A` with memory at `K`. (AGCIS pp.20–22)
- **TS K / TSK / TSO:** `ref/instr/ts_k.md` — Transfer to `K` variants; `TS K` writes `A` to `[K]` and may skip on overflow/underflow. (AGCIS pp.24–28)
- **MSK K:** `ref/instr/msk_k.md` — Mask `A` with `[K]` (bitwise AND semantics). (AGCIS pp.28–29)
- **MP K:** `ref/instr/mp_k.md` — Multiply `A` by `[K]`, place 32-bit product in `A:Q`. (AGCIS pp.46–60)
- **DV K:** `ref/instr/dv_k.md` — Divide `A:Q` by `[K]` using restoring division; quotient/remainder semantics. (AGCIS pp.61–80)

Notes:
- Files marked above may not yet exist; existing per-instruction docs are available at `ref/instr/` for CCS, SU, MP, and DV.
- If you prefer a different file naming or mnemonic casing, I can rename them globally.

Citations: primary source `agcis_2_machine_instructions.pdf` (see page ranges above per entry).