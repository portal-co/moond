# AGCIS Issue 2 — Machine Instructions (modernized)

Source: `agcis_2_machine_instructions.pdf`

Note: this document modernizes and reframes the original AGCIS Issue 2 descriptions (pages cited below). Changes made:
- Removed the original "subinstruction" framing and present each machine instruction as a single, ordered micro-op sequence (micro-ops correspond to the original control-pulse Actions).
- Reframed hardware constructs into modern assembly-style concepts (registers, micro-ops, memory read/write, condition flags).
- Octal numbers use `0o` prefix (originally printed in octal, e.g. `0020` → `0o20`).

## Summary (what changed)
- **Subinstructions → Micro-ops:** Each original subinstruction (12 Actions) is represented as a sequence of micro-ops executed in order. The second subinstruction that performed housekeeping (STD2) is merged into the instruction's micro-op stream so every instruction is self-contained.
- **Registers:** Kept original register names (`A`, `B`, `C`, `G`, `P`, `Z`, `Q`, `S`, `LP`, `R`, etc.) but describe them as standard registers with bit-width and conventional names (e.g., `A: accumulator`, `Z: PC`).
- **Memory model:** F/E memories are presented as a single address space with addresses given in octal using `0o` prefix; addresses `>= 0o20` are F/E memory words, `<= 0o17` index flip-flop registers (cited below).

## 1. Introduction / Execution model (modernized)
- Each machine instruction is a sequence of Actions produced by the Sequence Generator (SQG). Actions occur at 1.024 MHz (every ≈0.977 µs). 12 Actions = one original subinstruction; an instruction may contain one or more subinstructions (we transform these into a linear micro-op stream).
- A micro-op corresponds to a control-pulse set generated at a specific Action. Micro-ops include: memory read, memory write, register read/write, parity test, flag tests (overflow/underflow/minus-zero), and control-flow operations.

Citation: AGCIS Issue 2, p.6, §2-1; p.6–7 §2-3–2-5.

## 2. Registers and conventions (modernized)
- `A` — primary accumulator (16 data bits + parity bit handled separately)
- `B` / `C` — buffer register (direct side `B`, complement side `C`)
- `G` — F/E memory data register (bit 0 parity, bits 1–15 data, bit 16 sign/overflow semantics)
- `P` — parity register / temporary flip-flop (parity gate target)
- `Z` — program counter (holds "next" address)
- `Q` — return address / saved PC for interrupts
- `S` — address staging register (used during memory selection)
- `LP` — loop pointer / special register for indexing

(Behavior and bit movement conventions derived from AGCIS Issue 2, pp.14–15, §§2-9–2-12.)

Citation: p.12–15, §§2-9–2-12.

## 3. Memory addressing & addressing categories
- Addresses are octal. Use `0o20` as the boundary between related memory regions:
  - `0o00`–`0o17`: flip-flop registers (special single-word registers)
  - `0o20`–`0o1777`: F/E memory word addresses
- During a read, the addressed word is placed in register `G` (micro-op: `read -> G`). During writes, `G` contents are written into memory at the addressed location.

Citation: p.15, §2-10.

## 4. Common micro-op group (modernized STMIC)
- Most instructions perform a common memory-inquiry cycle (STMIC) which we model as a fixed micro-op subsequence used by many instructions:
  1. `RZ` — read `Z` into write amplifiers (stage: fetch `z = Z`)
  2. `WS` — reset `S`, write `z` into `S`
  3. `WY` — clear Adder inputs X/Y; write `z` into Y
  4. `CI` — force carry in adder (effectively add 1)
  5. adder produces `z+1` and that value is used to update `Z` later
  6. If `S >= 0o20` then `G <- MEM[S]` (Action 6) — read memory word into `G`
  7. `RG`, `WB`, `WP` — transfer `G` -> `B` and `P` as needed (prepare next instruction word)
  8. parity test `TP` may be applied to `P`

- In this modernization we combine these into a micro-op macro `STMIC` that can be inlined into instruction sequences rather than split across subinstructions.

Citation: p.14–15, §§2-9–2-11; table summary in pp.7–11.

## 5. Example: STD2 (reframed as an inlined micro-op sequence)
Original STD2 illustrated standard housekeeping (update PC, stage next instruction fetch). Modernized micro-op stream (with Actions mapped):

- Context: executing instruction at `L`, original AGC stores `z = c(Z) = L + 1` and uses subsequent pulses to produce `z+1 = L + 2` for the updated PC.

Micro-op sequence (all numbers octal when shown):

1. micro-op: `FETCH_Z` — `z = Z` (RZ) [AGCIS p.13–14, §2-9]
2. micro-op: `S_WRITE(z)` — `S <- z` (WS)
3. micro-op: `CLEAR_ADDER_AND_LOAD_Y(z)` — `X,Y <- 0; Y<-z` (WY)
4. micro-op: `FORCE_CARRY_AND_ADD_ONE` — trigger carry to adder so result = z + 1 (CI)
5. micro-op: `UPDATE_Z` — `Z <- z+1` (RU + WZ)  [now Z holds L+2]
6. micro-op: `MEM_SELECT(S)` — if `S >= 0o20` then `G <- MEM[S]` else address a flip-flop register (per address range rules)
7. micro-op: `STMIC_SETUP` — `B <- G`; `P <- parity(G)`; `WP/WB` as appropriate

Citation: example flow and figure: p.12–14, figure 2-1 and §§2-9–2-11.

## 6. Parity and flags (modernized behavior)
- Parity is always tracked in the hardware; `GP` gates the parity bit into position 0 of `G` if needed. Parity test (`TP`) triggers an alarm on incorrect parity (original issue assumed correct parity, but the test remains in hardware).
- Overflow/underflow tests map to modern `OF`/`UF` flags (set when test micro-ops detect conditions and delivered to interrupt inputs per original design).

Citation: table and description of control pulses, pp.7–11, table 2-1.

## 7. Conventions used in this modernized doc
- Octal numbers: `0oNNN` (e.g. `0020` → `0o20`)
- Micro-ops: short, explicit steps mapping to control pulses and Actions
- Control-pulse names preserved (RZ, WS, WY, CI, RG, WB, WP, TP, GP, RU, WZ, etc.) to make traceability to original doc easy

## 8. Next steps and open items
- I will read the next chunk of pages (16–40) and continue extracting instruction descriptions and transforming them into modern micro-op sequences and individual Markdown instruction pages.
- Question: For the modernized assembly format, do you prefer a specific syntax (suggested default below) for instruction examples?

Suggested micro-op pseudo-assembly format:

```
; Example: STD2 (inlined)
FETCH_Z        ; RZ
S_WRITE z      ; WS
CLEAR_ADDER    ; WY
FORCE_CARRY    ; CI (z+1 produced)
UPDATE_Z z+1   ; RU + WZ
MEM_READ S     ; if S >= 0o20: G <- MEM[S]
PREP_NEXT      ; RG, WB, WP to stage next word
```

If you approve this style, I'll keep creating separate Markdown files per source PDF and per instruction group, each with explicit citations (pdf page and paragraph identifier where available).
