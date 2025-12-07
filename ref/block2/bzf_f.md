# BZF F — Branch on Zero to Fixed F (Block-2)

Summary
- Test register A; if zero, take next instruction from F (fixed); otherwise continue. Requires EXTEND to set EXT bit for extra addressing.

Detailed pseudocode

void BZF_F(uint16_t F) {
    // Standard memory inquiry
    STMIC_stage();

    // If A == 0 take next instruction from fixed F, otherwise continue
    if (A == 0) {
        // Branch taken: load fixed-field instruction from F on next fetch
        S = F;
        // NOTE: when EXT bit is required the EXTEND instruction must have been executed
        // prior to this instruction to set SQ.EXT; derived order code is read by STD2 at time 12.
        STD2_execute(); // finalize and call forward (covers write/restore timing)
    } else {
        // No-branch path: keep normal sequencing and finalize
        STD2_execute();
    }
}

Notes
- EXT handling: callers must execute EXTEND() when an Extra-Code/Fixed-F instruction requires SQ.EXT to be set before BZF_F.
- This pseudocode models the observable behavior; STD2_execute() encapsulates the STD2 subinstruction timing and G/S/B/SQ load semantics.

Inline notes
- BZF_F in Block-2 often relies on EXTEND being executed immediately prior; inlining STMIC makes the dependency clearer. See ref/definitions/EXTEND.md and ref/cpu/registers.md for canonical behavior.

Edge cases / TODOs
- Exact timing requirement for EXT bit relative to STD2: TODO:VERIFY.
- Behavior if EXTEND is omitted but operand expects EXT semantics: TODO:VERIFY."