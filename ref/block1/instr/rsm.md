# RSM — Resume Program (modernized)

Source: `agcis_2_machine_instructions.pdf` — pp.89–91, fig. 2‑36.

Summary
- Operation: Restore the saved program context (ZRUPT/BRUPT) back into `Z` and `B`, release the interrupt-in-progress inhibition, and continue execution at the restored instruction.
- Modernization: Presented as a single routine (NDX0 + RSM in the original) that atomically restores and resumes.

Micro-op (C-like pseudocode)

```c
void RSM(void) {
    // Restore saved registers
    B = BRUPT;
    Z = ZRUPT;

    // Clear interrupt-in-progress so new interrupts may occur
    set_interrupt_in_progress(false);

    // Load the order code of the restored B and execute
    SQ = extract_order_code(B);
}
```

Notes
- The original RSM is executed via `NDX 0025` (special NDX) and a short microsequence; this routine captures the intended high-level semantics for emulator documentation.

Citations
- AGCIS Issue 2, pp.89–91, fig. 2‑36.
Inline notes
- Block-1 uses canonical helper references in ref/definitions and ref/cpu/registers.md; where SCALER or other substantial refs are used, provide citations or mark TODO:VERIFY if uncertain.

Edge cases / TODOs
- TODO:VERIFY uncertain external references (SCALER etc.) — provide citation backup or mark as training-derived.

Audit
- Scanned repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative support; if evidence exists it is noted here. Initial audit: authoritative support not found in repo PDFs or ambiguous/OCR-unclear, so this file retains `TODO:VERIFY` and is provisionally marked as "inferred from training/model" when applicable.
- Action: retain `TODO:VERIFY` marker and consult ref/TODO_AUDIT.md for central tracking. If additional AGC memos or hardware logs are available, add citations below or update this Audit block.

Audit resolution (2025-12-07T08:34:19.588Z):
- Targeted sources reviewed: AGCIS Issue 2 (ref/moon/agcis_2_machine_instructions.pdf) pages 15–36, 46–60, 61–80, 86–102; AGCIS Issue 3 (ref/moon/agcis_3_central_processor.pdf) pages 3–11; AEAProgrammingReference.pdf pages 15–18 where applicable.
- Behavior matching these sources is considered supported and marked resolved in-file when specific; remaining ambiguous details retain TODO:VERIFY and are listed in ref/TODO_AUDIT.md for later authoritative sourcing.
