# AGCIS Issue 2 — Machine Instructions (instructions, modernized)

See the canonical per-instruction index: `agcis_2_machine_instructions_index.md`.

Per-instruction docs have been moved to `ref/instr/` (one file per instruction). Existing files include:

- `ref/instr/ccs_k.md`  — CCS K
- `ref/instr/su_k.md`   — SU K
- `ref/instr/mp_k.md`   — MP K
- `ref/instr/dv_k.md`   — DV K
- `ref/instr/tc_k.md`   — TC K
- `ref/instr/xch_k.md`  — XCH K
- `ref/instr/csk_k.md`  — CSK K
- `ref/instr/ts_k.md`   — TS / TSK / TSO
- `ref/instr/msk_k.md`  — MSK K
- `ref/instr/ad_k.md`   — AD K
- `ref/instr/ndx_k.md`  — NDX K

Source: `agcis_2_machine_instructions.pdf` (pages 16–40)
- Micro-ops map to original control pulses and Actions (RZ, WS, WY, CI, RG, WB, WP, TP, GP, RU, WZ, etc.).
- `STMIC` denotes the common memory-inquiry micro-op group (fetch/address staging/prepare next instruction) referenced in AGCIS.

---

## Instruction: TC K (Transfer Control to K)
- Order code: `0`
- Behavior (modernized): Set the next instruction to the instruction located at `K` and save the current next address in `Q` so execution can return.

Micro-op stream (inlined STD behavior):
1. `STMIC` ; prepare `G`, `B`, `P` from `S` (S is loaded from B which holds K)  (AGCIS p.18, §2-16–2-18)
2. `Q <- Z` (save current next address z into `Q`)  (p.18)
3. `Z <- B + 1` (compute and write (B + 1) into `Z`) — this becomes new PC (p.18–19)
4. `SQ <- ordercode(G)` ; set execution to the order code of the fetched instruction at `K` (p.19)

Asm example:
```
; TC K
TC K   ; transfer control to K (save Z -> Q, set Z <- K+1)
```

Citation: AGCIS Issue 2, pp.18–19, §§2-16–2-18.

---

## Instruction: XCH K (Exchange A with location K)
- Order code: `3`
- Behavior: Exchange the contents of accumulator `A` with memory at `K`. For F-memory locations the memory content is placed into `A` and the content at `K` may be overwritten by `A` depending on addressing (flip-flop vs memory) and special codes. Bit movement rules for sign/overflow bits apply (see conventions).

Micro-op stream:
1. `STMIC` (S <- B.address; fetch G <- MEM[S])  (p.20–21)
2. `P <- A` ; store parity of `A` into `P` (temp) (p.21)
3. `TEMP <- A` ; `A <- G.data` ; `G <- TEMP` ; handle parity gating (GP/TP) (p.21)
4. `Z <- Z + 1` ; continue with next instruction (inlined STD) (p.21)

Asm example:
```
; XCH K
XCH K   ; swap A <-> [K]
```

Citation: AGCIS Issue 2, pp.20–22, §§2-19–2-21.

---

## Instruction: CSK (Clear A and complement data from K)
- Order code: `4`
- Behavior: Load the complemented value of memory at `K` into `A` (A := ~[K]), restore K as required, then continue.

Micro-op stream:
1. `STMIC` (fetch G <- MEM[S]) (p.22–23)
2. `A <- complement(G)` (apply complement; parity handled) (p.23)
3. `Z <- Z + 1` (inlined STD) (p.23)

Asm example:
```
CSK K  ; A := ~[K]; next
```

Citation: AGCIS Issue 2, pp.22–23, §§2-22–2-24.

---

## Instruction: TSK / TSO (Transfer to K variants) and TS K (Transfer Data to K / Skip on overflow)
- Order code: `5` (TSK/TSO are transfer-to-K variants); `TS K` transfers A to memory at `K` with conditional skip when overflow/underflow is present.

Modernized behavior (TS K):
- If `A` has no overflow/underflow, write `A` to `K` and continue normally.
- If `A` has overflow, set `A` to `0o1` and skip the following instruction (advance PC by two instead of one). If underflow, set `A` to `0o177776` and skip similarly.

Micro-op stream:
1. `STMIC` ; staging (p.24–26)
2. `TEMP <- A` ; `P <- parity(A)` ; test overflow/underflow (TOV) (p.25–26)
3a. if no OF/UF: `MEM[S] <- A` ; `Z <- Z + 1`
3b. if OF: `A <- 0o1` ; `Z <- Z + 2`
3c. if UF: `A <- 0o177776` ; `Z <- Z + 2`

Asm example:
```
TS K    ; [K] := A ; if OF/UF then A := +1/-1 and skip next
```

Citation: AGCIS Issue 2, pp.24–28, §§2-25–2-28 and figures 2-6/2-7.

---

## Instruction: MSK K (Mask A with memory at K)
- Order code: `7`
- Behavior: A := A AND [K]; restore or write-back to K per addressing. Implemented via OR/complement trick in original hardware; we present it as a bitwise AND.

Micro-op stream:
1. `STMIC` ; fetch [K] -> G (p.28–29)
2. `A <- A AND G` (replace original OR/complement approach with direct logical AND in semantics) (p.29)
3. `Z <- Z + 1`

Asm example:
```
MSK K   ; A := A & [K]
```

Citation: AGCIS Issue 2, pp.28–29, §§2-30–2-32.

---

## Instruction: AD K (Add memory K into A; update OVCTR on overflow/underflow)
- Order code: `6`
- Behavior: Perform `A := A + [K]` (arithmetic), and if overflow/underflow occurs, adjust OVCTR (via PINC/MINC) before proceeding.

Micro-op stream:
1. `STMIC` ; fetch `[K]` into `G` and stage `B/P` (p.31–32)
2. `SUM <- A + G` ; arithmetic performed via adder (CI and related pulses) (p.31)
3. `A <- SUM` ; test overflow/underflow (WOVI)
4a. if OF: schedule `PINC` to increment OVCTR (p.32–33)
4b. if UF: schedule `MINC` to decrement OVCTR
5. `Z <- Z + 1`

Asm example:
```
AD K    ; A := A + [K] ; OVCTR updated on overflow/underflow
```

Citation: AGCIS Issue 2, pp.31–33, §§2-33–2-35 and figure 2-9.

---

## Instruction: NDX K (Index Next Instruction)
- Order code: `2`
- Behavior: Index the *next* instruction (`B`) by adding the contents of `K` to the instruction word at `z = L+1`, producing a modified instruction to execute next. When K == `0o25` this is treated as RESUME/RSM behavior.

Micro-op stream:
1. `STMIC` ; fetch `K` into `B`, check `K != 0o25` via TRSM (p.33–34)
2. `Z` and `B` arithmetic: compute `B' := instruction_at(z) + [K]` (p.34–36)
3. `B <- B'` ; `SQ <- ordercode(B')` ; set `Z <- Z + 2` (normally) or to computed address when both components were TC and no carry overflow (special case p.36)

Asm example:
```
NDX K  ; index next instruction by [K]
```

Citation: AGCIS Issue 2, pp.33–36, §§2-36–2-39 and figures 2-10, 2-11.

---

## Instruction: CCS K (Count, Compare, Skip with Data at K)
- Order code: `1`
- Behavior: Compare `K` against zero and choose the next instruction based on sign and zero-state:
  - if `c(K) > +0` → next is `L+1`
  - if `c(K) = +0` → next is `L+2`
  - if `c(K) < -0` → next is `L+3`
  - if `c(K) = -0` → next is `L+4`

Also updates `A` as described in original doc (A := c(K) - 1 in many cases).

Micro-op stream:
1. `STMIC` ; fetch `G <- [K]` and place into `B/P` (p.36–38)
2. examine `G` sign/zero and set `A` accordingly (cases for +0, -0, and magnitude ranges) (p.36–40)
3. set `Z` to `L+N` where `N` is 1–4 based on the case; restore `K` as necessary

Asm example:
```
CCS K   ; compare [K] with zero; A := [K] - 1 (or 0); branch to L+1..L+4
```

Citation: AGCIS Issue 2, pp.36–40, §§2-40–2-42 and figures 2-12..2-14.

---

## Notes / traceability
- Each micro-op listed above is traceable to the original control pulses / Actions in the AGCIS diagrams and text. I preserved control-pulse names in comments to aid verification.
- When the original specified separate subinstructions (e.g., XCH0 + STD2), I inlined STD behavior so each instruction file stands alone.

## Next actions
- I'll continue reading the next PDF chunks (remaining pages of this PDF and then other AGCIS PDFs) and produce per-instruction Markdown files for each group.
- Please confirm whether you prefer a single-per-instruction Markdown file (e.g., `ref/instr/ADK.md`) or grouped files (like this one). If you have preferred micro-assembly style changes (different mnemonic names or syntax), tell me and I'll adapt future files accordingly.
