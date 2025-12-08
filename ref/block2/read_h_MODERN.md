# READ H — Read I/O Channel (Block-2)

Source: `agcis_32_blk2_instructions.pdf` — pages 151-153 (section 32-219, figure 32-39).

## Summary

Read the content of I/O channel H into register A. Channels connect the AGC to external devices and systems.

**Operation:** `A = channel[H]`

**Channel range:** 0o0-0o177 (7-bit channel address, 128 channels)

## Pseudocode

```c
void READ_H(uint16_t H) {
    // Extract channel number from instruction
    uint8_t channel_num = H & 0o177;  // 7-bit channel address
    
    // Read from I/O channel
    // (Channel hardware provides 15-bit data, bit 15 is sign)
    uint16_t channel_data = io_channels[channel_num];
    
    // Store in accumulator
    A = channel_data & 0o77777;  // Mask to 15 bits
    
    // Branch to next instruction (STD2 completion)
    // See ref/definitions/STD2.md for canonical subinstruction
    Z = Z + 1;
    uint16_t next = memory[Z];
    SQ = extract_order_code(next);
}
```

## Notes

### I/O Channel System

The AGC has 128 I/O channels (0o0-0o177) connecting to:
- **Inertial Measurement Unit (IMU):** Gyroscopes and accelerometers
- **Optics:** Telescope and sextant data
- **Radar:** Rendezvous and landing radar
- **DSKY:** Display and keyboard interface
- **Guidance computer:** Counters and timers
- **RCS:** Reaction control system jets
- **AGS:** Abort guidance system interface

### Channel Data Format

Most channels provide 15-bit signed data:
- **Bit 15:** Sign bit
- **Bits 14-1:** Magnitude (14 bits)
- **Bit 0:** Parity (generated on read)

Some channels use different formats (documented in AEA Programming Reference).

### Read vs Write Channels

- **READ H:** Read input channel (sensor data, status registers)
- **WRITE H:** Write output channel (control signals, displays)
- Some channels are read-only, some write-only, some bidirectional

### Channel Address Encoding

The H operand is a 7-bit channel address:
- Instruction encoding: bits 0-6 of operand
- Channel hardware: 128 addressable locations
- Channel map: See AEA Programming Reference pages 15-18

## Citations

- AGCIS Issue 32 (Block-2), pages 151-153
  - Section 32-219: READ H instruction description
  - Figure 32-39: READ H timing diagram
- Channel system overview: AGCIS Issue 3, pages 30-35
- Channel addresses: AEA Programming Reference, pages 15-18 (PGNS I/O channels)

## Audit

- PDF source: AGCIS Issue 32 pages 151-153 document READ H operation
- Channel addressing: 7-bit address (0o0-0o177) verified in section 32-219
- I/O system: Overview in AGCIS Issue 3 pages 30-35
- Channel map: AEA Programming Reference pages 15-18 provide channel assignments
- Data format: 15-bit with sign bit (standard for most channels)
- Status: Core behavior verified; specific channel formats in AEA reference

---

**Modernization Note:** This file uses unified modern pseudocode style (2025-12-08). Hardware pulse names removed. STD2 inlined with reference. I/O system described at behavioral level.
