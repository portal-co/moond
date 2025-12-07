# BZF F — Branch on Zero to Fixed F (Block-2)

Summary
- Test register A; if zero, take next instruction from F (fixed); otherwise continue. Requires EXTEND to set EXT bit for extra addressing.

Pseudocode

void BZF_F(uint16_t F) {
    STMIC_stage();

    // Branch when A == 0
    if (A == 0) {
        S = F; // take next instruction from F
    } else {
        // Normal path: proceed to STD2
        STD2_execute();
    }
}

Notes
- Block-2 behavior mirrors Block-1; the `EXTEND` step is modeled outside this function (caller should set SQ EXT bit when required)."