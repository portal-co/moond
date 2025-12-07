# SHINC — Shift Content of Addressed Counter (modernized)

Source: `agcis_2_machine_instructions.pdf` — pp.92–95, figs. 2‑39..2‑40.

Summary
- Operation: Shift the addressed counter left by one position (multiply by two) with sign handling. Used for converting serial uplink/radar-range data into parallel words.
- Modernization: Single routine implementing the shift, sign/overflow handling, and program-interrupt signaling when a flagged bit is shifted into the test position.

Micro-op (C-like pseudocode)

```c
void SHINC(void) {
    uint16_t ctr_addr = counter_priority_address();

    // Read counter as signed 16-bit (15-value bits + sign)
    int16_t e = (int16_t)(MEM[ctr_addr] & 0xFFFF);

    // Shift left by one (logical on value bits, preserve sign rules per AGC)
    int32_t shifted = (int32_t)e << 1;

    // Handle overflow: reverse sign bit if required and prevent end-around carry
    if (shifted > 0x7FFF || shifted < -0x8000) {
        shifted = reverse_sign_bit(shifted);
        notify_shift_overflow(ctr_addr);
    }

    // Store back lower 16 bits with parity
    uint16_t out = (uint16_t)(shifted & 0xFFFF);
    MEM[ctr_addr] = out | (compute_parity_bit(out) << 15);

    // If bit 15 (flag) was set before the shift, signal program priority (UPRUPT)
    if (flag_bit_was_set(e)) trigger_uprupt();

    clear_counter_priority_request();
}
```

Notes
- `flag_bit_was_set()` models the check of bit position 15 described in the AGCIS text (used to indicate end-of-uplink-word).
- The hardware uses several micro-action pulses; this routine focuses on logical result and side-effects (UPRUPT signaling, overflow handling).

Citations
- AGCIS Issue 2, pp.92–96, figs. 2‑39..2‑40.
Inline notes
- Block-1 uses canonical helper references in ref/definitions and ref/cpu/registers.md; where SCALER or other substantial refs are used, provide citations or mark TODO:VERIFY if uncertain.

Edge cases / TODOs
- TODO:VERIFY uncertain external references (SCALER etc.) — provide citation backup or mark as training-derived.

Audit
- Scanned repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative support; if evidence exists it is noted here. Initial audit: authoritative support not found in repo PDFs or ambiguous/OCR-unclear, so this file retains `TODO:VERIFY` and is provisionally marked as "inferred from training/model" when applicable.
- Action: retain `TODO:VERIFY` marker and consult ref/TODO_AUDIT.md for central tracking. If additional AGC memos or hardware logs are available, add citations below or update this Audit block.

Audit update (2025-12-07T08:25:31.750Z): Repository PDF ref/moon/agcis_2_machine_instructions.pdf (selected pages cited in file headers) contains corroborating descriptions for the following behaviors: AD K overflow handling (PINC/MINC), NDX/EXTEND and STD2/XCH semantics (overflow bit preservation and STD2 sequencing), SHINC/SHANC shift-and-flag semantics, MP subinstruction sequencing and DV edge-case handling. Where the file documents one of these behaviors the TODO:VERIFY has been considered supported by the PDF and may be cleared later after source citation; remaining ambiguous items are retained as TODO:VERIFY. See ref/TODO_AUDIT.md for central tracking.

Audit resolution (2025-12-07T08:30:24.624Z):
- Supported by AGCIS Issue 2 (FR-2-102A) — selected pages read: 15–36, 46–60, 61–80, 86–102 which document instruction semantics (TC/XCH/STD2, AD/SU and OVCTR handling via PINC/MINC, NDX/EXTEND, MP and DV subinstruction sequencing, SHINC/SHANC behavior).
- Corroborating CPU/register behavior in AGCIS Issue 3 (FR-2-103A) pages 3–11 (register transfers, bit-15/16 movement, adder end-around carry and parity behavior).
- AEAProgrammingReference.pdf pages 15–18 provide PGNS scaler/register formats where applicable.
- Status: TODO:VERIFY items related to these behaviors are marked as resolved (supported by the cited PDFs); remaining ambiguous items remain TODO:VERIFY.
