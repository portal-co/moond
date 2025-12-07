# GO — Computer GO (modernized)

Source: `agcis_2_machine_instructions.pdf` — pp.100–101 (section 2‑121..2‑123).

Summary
- Operation: Start the computer by loading and executing the instruction at a predefined start address (02030 in AGC examples). Equivalent in effect to `TC` but uses the start-read control pulse to fetch the start address.

Micro-op (C-like pseudocode)

```c
void GO(void) {
    // Start address is provided by hardware (start register)
    uint16_t start_addr = start_address_from_hardware(); // typically 0o2030

    // Stage & fetch the start instruction
    Z = start_addr;
    B = MEM[start_addr] & 0x7FFF;
    SQ = extract_order_code(B);
}
```

Notes
- GO is functionally equivalent to `TC` but uses the `RSTRT` pulse to obtain the start address. It leaves the machine ready to execute the start instruction.

Citations
- AGCIS Issue 2, pp.100–101.
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
