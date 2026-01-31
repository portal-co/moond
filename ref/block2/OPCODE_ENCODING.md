# Block-2 Opcode Encoding Overview

> Created: 2026-01-31T01:22:00.000Z  
> Updated: 2026-01-31T02:00:00.000Z (verified encoding with PDF, fixed address field sizes)
>
> Source: `agcis_32_blk2_instructions.pdf` — pages 10–24 (tables 32-2, 32-3), 93–106 (NDX, AD, SU, MP), 163–170 (EXTEND and extracode instructions).
>
> **Verification Status**: Encoding verified against PDF for TC, CA, CS, AD, MP, EXTEND, GO, and extracode operation. Address field sizes confirmed for 3-bit, 6-bit whole codes, and quarter codes.

## Overview

This document describes the instruction encoding format for AGC Block-2 machine instructions. Block-2 instructions are encoded as 16-bit words with a 15-bit data field (bits 1-15) and a parity bit (bit 0). The instruction format uses octal encoding naturally aligned to 3-bit groups.

## Instruction Word Format

```
Bit:  0  | 1 2 3 | 4 5 6 | 7 8 9 | 10 11 12 | 13 14 15
      P  | Opcode|      Address/Operand Field         |
```

- **Bit 0**: Parity bit (not shown in octal representation)
- **Bits 1-3**: Primary opcode (octal digit)
- **Bits 4-15**: Address or operand field (4 octal digits)

## Instruction Types and Encoding

### Basic Instruction Categories

Block-2 instructions are categorized by their address mode and operation type:

- **K-type**: Direct address to erasable/fixed memory (12-bit address field)
- **E-type**: Extended address (erasable or E-bank memory)
- **F-type**: Fixed memory address
- **C-type**: Counter/peripheral address
- **H-type**: I/O channel address

### Order Code Format

Instructions use either:
- **Whole order codes**: 3 octal digits (bits 1-9), e.g., `00.`, `01.`, `06.`
- **Quarter order codes**: 4 octal digits (bits 1-12), e.g., `05.0`, `05.4`, `02.2`

The format `XX.Y` represents:
- `XX` = Primary opcode (bits 1-6, 2 octal digits)
- `Y` = Sub-opcode (bits 7-9, 1 octal digit)

For quarter codes `XX.YZZ`:
- `XX.Y` = Primary + sub-opcode (bits 1-9, 3 octal digits)
- `ZZ` = Further qualification (bits 10-12, 2 octal digits)

## Regular Instructions

### Sequence Changing Instructions

| Mnemonic | Order Code | Type | Description |
|----------|------------|------|-------------|
| TC       | 00.        | K    | Transfer Control to K |
| TCF      | 01.2, 01.4, 01.6 | F | Transfer Control to Fixed F |
| CCS      | 01.0       | E    | Count, Compare, and Skip on E |
| BZF      | 16.2, 16.4, 16.6 | F | Branch on Zero to Fixed F |
| BZMF     | 12.2, 12.4, 12.6 | F | Branch on Zero or Minus to Fixed F |

### Fetching and Storing Instructions

| Mnemonic | Order Code | Type | Description |
|----------|------------|------|-------------|
| CA       | 03.        | K    | Clear and Add K |
| CS       | 04.        | K    | Clear and Subtract K |
| DCA      | 13.        | K    | Double Clear and Add K |
| DCS      | 14.        | K    | Double Clear and Subtract K |
| TS       | 05.4       | E    | Transfer to Storage E |
| XCH      | 05.5       | E    | Exchange A and E |
| LXCH     | 02.2       | E    | Exchange L and E |
| QXCH     | 12.2       | E    | Exchange Q and E |
| DXCH     | 05.2       | E    | Double Exchange A and E |

### Modifying Instructions

| Mnemonic | Order Code | Type | Description |
|----------|------------|------|-------------|
| NDX      | 05.0 (E), 15. (K) | E/K | Index with E or K |

### Arithmetic and Logic Instructions

| Mnemonic | Order Code | Type | Address Bits | Description | Requires EXTEND |
|----------|------------|------|--------------|-------------|-----------------|
| AD       | 06.        | K    | 12-bit (bits 4-15) | Add K | No |
| SU       | 16.0       | E    | 10-bit (bits 7-15 minus quarter) | Subtract E | Yes |
| MP       | 17.        | K    | 9-bit (bits 7-15) | Multiply by K | Yes |
| DV       | 11.0       | E    | 10-bit (bits 7-15 minus quarter) | Divide by E | Yes |
| ADS      | 02.6       | E    | 10-bit | Add to Storage E | Yes |
| DAS      | 02.0       | E    | 10-bit | Double Add to Storage E | Yes |
| INCR     | 02.4       | E    | 10-bit | Increment E | Yes |
| AUG      | 12.4       | E    | 10-bit | Augment E | Yes |
| DIM      | 12.6       | E    | 10-bit | Diminish E | Yes |
| MSU      | 12.0       | E    | 10-bit | Modular Subtract E | Yes |
| MSK      | 07.        | K    | 12-bit (bits 4-15) | Mask with K | No |

**Note**: Instructions with 6-bit order codes (like MP 17.) have only 9 bits remaining for address, limiting range to 0-777 octal (0-511 decimal). This is sufficient for E-memory and CP registers but cannot address all of F-memory directly.

### Channel Instructions

| Mnemonic | Order Code | Type | Description |
|----------|------------|------|-------------|
| READ     | 10.0       | H    | Read H |
| WRITE    | 10.1       | H    | Write H |
| RAND     | 10.2       | H    | Read and AND H |
| WAND     | 10.3       | H    | Write and AND H |
| ROR      | 10.4       | H    | Read and OR H |
| WOR      | 10.5       | H    | Write and OR H |
| RXOR     | 10.6       | H    | Read and Exclusive OR H |

### Special Instructions

| Mnemonic | Order Code | Type | Description |
|----------|------------|------|-------------|
| EXTEND   | 00.0006    | -    | Extend (next instruction is extra-code) |
| INHINT   | 00.0004    | -    | Inhibit Interrupt |
| RELINT   | 00.0003    | -    | Release Inhibit Interrupt |
| RESUME   | 05.0017    | -    | Resume Interrupted Program |
| CYR      | .0020      | -    | Cycle Right (via register) |
| SR       | .0021      | -    | Shift Right (via register) |
| CYL      | .0022      | -    | Cycle Left (via register) |
| EDOP     | .0023      | -    | Edit Operator (via register) |

## Involuntary Instructions

### Interrupting Instructions

| Mnemonic | Order Code | Type | Description |
|----------|------------|------|-------------|
| RUPT     | 10.        | -    | Interrupt Program Execution |
| GO       | 00.4000    | -    | Go (restart at E-memory 04000) |

### Counter Instructions

| Mnemonic | Order Code | Type | Description |
|----------|------------|------|-------------|
| PINC     | none       | C    | Plus Increment C |
| MINC     | none       | C    | Minus Increment C |
| DINC     | none       | C    | Diminish Increment C |
| PCDU     | none       | C    | Plus Counter Down Up C |
| MCDU     | none       | C    | Minus Counter Down Up C |
| SHINC    | none       | C    | Shift Increment C |
| SHANC    | none       | C    | Shift and Add Increment C |

**Note**: Counter instructions have no explicit order codes; they are triggered by hardware conditions and counter address settings.

## Peripheral Instructions

| Mnemonic | Order Code | Type | Description |
|----------|------------|------|-------------|
| TCSAJ    | 00.        | K    | Transfer Control to Specified Address K |
| FETCH    | none       | K    | Fetch K (displays on GSE) |
| STORE    | none       | E    | Store E (loads from GSE) |
| INOTRD   | none       | H    | I/O Not Read H (displays on GSE) |
| INOTLD   | none       | H    | I/O Not Load H (loads from GSE) |

**Note**: Peripheral instructions are controlled by Ground Support Equipment (GSE).

## Extended Instructions (Extra-Code)

Instructions preceded by EXTEND (00.0006) use the extended instruction set. When EXTEND is executed, it sets the SQ-EXT bit, causing the next instruction to be interpreted as an extra-code instruction. This allows many E-type and extended-function instructions.

### EXTEND Instruction

**Order Code:** 00.0006  
**Execution:** Executes STD2 subinstruction (1 MCT)  
**Effect:** Sets bit position SQ-EXT to ONE in register SQ

The next instruction fetched after EXTEND is interpreted as an extra-code instruction. The EXTEND instruction itself completes with a standard STD2 subinstruction that increments Z and fetches the next instruction.

### Extended Instruction Recognition

The Sequence Generator (SQG) determines which subinstruction to execute based on:
- **Regular instructions**: Determined by bits EXT through 13 (whole codes) or EXT through 11 (quarter codes)
- **Extended instructions**: Same bit positions but with SQ-EXT bit set to ONE
- **Channel instructions**: Determined by bits EXT through 10

### Instructions Requiring EXTEND Prefix

The following instructions **must** be preceded by EXTEND:

#### E-Type Extended Instructions (Extracode)

| Mnemonic | Order Code | Description |
|----------|------------|-------------|
| CCS      | 01.0       | Count, Compare, and Skip on E |
| TS       | 05.4       | Transfer to Storage E |
| XCH      | 05.5       | Exchange A and E |
| LXCH     | 02.2       | Exchange L and E |
| QXCH     | 12.2       | Exchange Q and E |
| DXCH     | 05.2       | Double Exchange A and E |
| NDX      | 05.0       | Index with E (basic instruction form) |
| SU       | 16.0       | Subtract E |
| DV       | 11.0       | Divide by E |
| ADS      | 02.6       | Add to Storage E |
| DAS      | 02.0       | Double Add to Storage E |
| INCR     | 02.4       | Increment E |
| AUG      | 12.4       | Augment E |
| DIM      | 12.6       | Diminish E |
| MSU      | 12.0       | Modular Subtract E |
| STORE    | none       | Store E (peripheral) |

#### Channel Instructions Requiring EXTEND

| Mnemonic | Order Code | Description |
|----------|------------|-------------|
| WAND     | 10.3       | Write and AND H |
| WOR      | 10.5       | Write and OR H |
| RXOR     | 10.6       | Read and Exclusive OR H |

**Note**: READ (10.0), WRITE (10.1), RAND (10.2), and ROR (10.4) do **not** require EXTEND.

### Extra-Code Instruction Sequence

```
EXTEND          ; Order code 00.0006 - sets SQ-EXT bit
XCH 0050        ; Order code 05.5050 - interpreted as extracode XCH E
; Next instruction executes normally (SQ-EXT cleared after extracode)
```

The EXTEND instruction:
1. Executes its own STD2 subinstruction
2. Sets the SQ-EXT flip-flop to ONE
3. Increments Z and fetches the next instruction
4. The next instruction is decoded with SQ-EXT=1, selecting the extracode subinstruction
5. After the extracode instruction completes, SQ-EXT is cleared

### Extracode Encoding Details

**Without EXTEND (Basic Instructions):**
```
Order Code: 05.0XXX  → NDX K (Index with K - uses bits 4-15 for address)
```

**With EXTEND (Extracode Instructions):**
```
EXTEND          → 00.0006 (sets SQ-EXT)
Order Code: 05.0XXX  → NDX E (Index with E - extracode interpretation)
```

The same order code bits are interpreted differently based on the SQ-EXT bit state. This effectively doubles the instruction set by providing an alternate interpretation for many opcodes.

### Extracode vs Basic Instruction Disambiguation

| Order Code Range | Without EXTEND | With EXTEND |
|------------------|----------------|-------------|
| 01.0             | BZF F          | CCS E       |
| 02.0             | LXCH E (basic) | DAS E       |
| 02.2             | LXCH E (basic) | LXCH E      |
| 02.4             | LXCH E (basic) | INCR E      |
| 02.6             | LXCH E (basic) | ADS E       |
| 05.0             | NDX K          | NDX E       |
| 05.2             | TS E (basic)   | DXCH E      |
| 05.4             | TS E (basic)   | TS E        |
| 05.5             | XCH E (basic)  | XCH E       |
| 10.3             | -              | WAND H      |
| 10.5             | -              | WOR H       |
| 10.6             | -              | RXOR H      |
| 11.0             | MP K (basic)   | DV E        |
| 12.0             | QXCH E (basic) | MSU E       |
| 12.2             | QXCH E (basic) | QXCH E      |
| 12.4             | BZF F (basic)  | AUG E       |
| 12.6             | BZF F (basic)  | DIM E       |
| 16.0             | BZF F (basic)  | SU E        |

**Note**: Some order codes have different meanings in basic vs extracode mode, while others are only accessible via extracode.

## Address Field Encoding

### K-Type Instructions (12-bit address)

Bits 4-15 contain a 12-bit address (4 octal digits):
- `0000-0007`: Central Processor registers
- `0010-1777`: Erasable memory (E-memory)
- `2000-7777`: Fixed memory (F-memory)

### E-Type Instructions (12-bit address)

Bits 4-15 contain a 12-bit address, typically:
- `0010-1777`: Erasable memory
- May use E-bank switching for extended erasable access

### F-Type Instructions (12-bit address)

Bits 4-15 contain a fixed memory address:
- `2000-7777`: Fixed memory address
- TCF and branch instructions use F-type

### H-Type Instructions (9-bit channel)

Bits 7-15 contain a 9-bit I/O channel address (3 octal digits)

### C-Type Instructions (9-bit counter)

Counter instructions reference counter addresses via hardware mechanisms rather than explicit instruction bits.

## Encoding Examples

### TC K (Transfer Control)
```
Order Code: 00.
Format: 00 XXX (where XXX = address K in octal)
Example: TC 04000  → 00.4000 → 0o004000
```

### AD K (Add)
```
Order Code: 06.
Format: 06 XXX (where XXX = address K in octal)
Example: AD 0100  → 06.0100 → 0o060100
```

### XCH E (Exchange)
```
Order Code: 05.5
Format: 05 5XX (where XX = address E in octal)
Example: XCH 0050  → 05.5050 → 0o055050
```

### READ H (Read Channel)
```
Order Code: 10.0
Format: 10 0HH (where HH = channel H in octal)
Example: READ 030  → 10.0030 → 0o100030
```

### EXTEND (Extended Instruction Prefix)
```
Order Code: 00.0006
Format: 00.0006
Binary: 0o000006
```

## Subinstruction Sequencing

Block-2 instructions execute as sequences of subinstructions:
- Most instructions: 1-3 subinstructions
- Division (DV): 7 subinstructions
- Each subinstruction (except DVO/DV4): 12 actions at 0.977 μsec = 11.7 μsec (1 MCT)

The stage counter (ST) and sequence register (SQ) control subinstruction execution:
- ST = 2: Execute STD2 (standard completion) subinstruction
- ST ≠ 2: Execute subinstruction based on SQ content

## Notes

- **Octal representation**: Order codes are traditionally written in octal (base-8) due to the natural 3-bit grouping of the AGC architecture.
- **Parity bit**: Bit 0 is used for odd parity across bits 1-16, but is not shown in order code notation.
- **Memory cycle time (MCT)**: 11.7 μsec per subinstruction (except divide operations).
- **Extra-code space**: EXTEND instruction allows access to additional instruction encodings by setting the SQ-EXT flip-flop.

## References

- See individual instruction files in `ref/block2/` for detailed behavioral descriptions.
- See `ref/definitions/STD2.md` for canonical subinstruction STD2 definition.
- See `ref/cpu/registers.md` for register definitions and types.
- See `ref/block2/differences.md` for Block-1 vs Block-2 behavioral differences.

---

Last updated: 2026-01-31T01:26:00.000Z
