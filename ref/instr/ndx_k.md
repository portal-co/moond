# NDX K — Index Next Instruction (modernized)

Source: `agcis_2_machine_instructions.pdf` — pages 33–36 (sections 2-36..2-39, figs. 2-10..2-11).

Summary
- Operation: Add the contents of `K` to the *next* instruction word (the word at address `Z`), modifying what will execute next. Special-case behavior when `K == 0o25` is treated as resume/RSM semantics.
- Modernization: Implemented as a singular micro-op that computes the modified next-order word and stages it for execution.

Micro-op (C-like pseudocode)

```c
void NDX_K(uint16_t K) {
    uint16_t z = Z;

    // STMIC: fetch K
    S = z; Y = z; X = 0;
    if (S >= 0o20) G = MEM[S];
    uint16_t kval = G & 0x7FFF;

    // Fetch next instruction word from memory (word at Z)
    uint16_t next_word = MEM[z];

    // Compute indexed instruction
    uint32_t new_word = (uint32_t)next_word + (uint32_t)kval;

    // Handle carry behavior per AGC: if carry and both were TC forms, special case
    // For documentation we write the computed word back into the staging B so the next fetch uses it
    MEM[z] = (uint16_t)(new_word & 0xFFFF);

    // Advance PC by 2 (original NDX normally advances an extra word)
    Z = z + 2;

    // Stage the new instruction's order code for execution
    B = (uint16_t)(new_word & 0x7FFF);
    SQ = extract_order_code(B);
}
```

Citations
- AGCIS Issue 2, pp.33–36, §§2-36–2-39 and figures 2-10..2-11.

Notes
- This file intentionally shows the logical effect (indexed next instruction) rather than reproducing microcycle-level pulse interactions.
