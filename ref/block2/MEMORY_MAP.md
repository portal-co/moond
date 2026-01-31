# AGC Block-2 Memory Map

> Created: 2026-01-31T06:06:19.068Z
>
> Source: `agcis_32_blk2_instructions.pdf`, `agcis_3_central_processor.pdf`

## Memory Address Ranges

The AGC Block-2 has a 12-bit address space (0-7777 octal, 0x0000-0xFFF hex) divided into:

### Central Processor Registers (00000-00007 octal)

The first 8 words of memory are mapped to CPU registers:

| Address (Octal) | Address (Hex) | Register | Description |
|-----------------|---------------|----------|-------------|
| 00000           | 0x0000        | A        | Accumulator |
| 00001           | 0x0001        | L        | Lower Accumulator |
| 00002           | 0x0002        | Q        | Return Address |
| 00003           | 0x0003        | EB       | E-Bank Register |
| 00004           | 0x0004        | FB       | F-Bank Register |
| 00005           | 0x0005        | Z        | Program Counter |
| 00006           | 0x0006        | BB       | B-Bank Register |
| 00007           | 0x0007        | (Zero)   | Fixed Zero Register |

**Note**: These addresses mirror the CPU registers. Transfer control (TC) to these addresses is nonsensical as they are not executable program memory.

### Erasable Memory (00010-03777 octal)

**Range**: 00010-03777 octal (0x0008-0x07FF hex)

Erasable (RAM) memory used for:
- Variables
- Temporary storage
- Stack
- Program state

**Note**: Executing TC to erasable memory is technically possible but highly unusual - most executable code resides in fixed memory.

### Fixed Memory (04000-77777 octal)

**Range**: 04000-77777 octal (0x0800-0x7FFF hex)

Fixed (ROM) memory containing:
- Executable program code
- Constants
- Lookup tables

**Special Address**: 04000 octal (0x0800 hex) is the restart address for the GO instruction.

## Instruction Address Validity

### TC K - Transfer Control to K

TC should only target **fixed memory** addresses (>= 04000 octal / 0x0800 hex).

- **Valid**: TC 04000, TC 10000, TC 77777 (fixed memory)
- **Invalid**: TC 00001 (register), TC 00100 (erasable memory)

This constraint resolves ambiguity between TC (order code 00.) and 6-bit opcodes:
- Word 0x0800 (04000 octal) decodes as **READ H** (010.0) with channel 0, NOT TC
- Word 0x0801 (04001 octal) decodes as **TC** with address 04001 (valid fixed memory)

### Channel Instructions (010.x)

Channel addresses are 6-bit (bits 10-15), resulting in words of the form:
- 010.0 00-77 → 040000-040077 octal (0x0800-0x083F hex)
- 010.1 00-77 → 044000-044077 octal (0x0900-0x093F hex)
- etc.

These word patterns overlap with TC's potential address space but are distinguished by:
1. **6-bit opcode takes precedence** over 3-bit opcode
2. **TC is only valid for addresses >= 04000** where it doesn't conflict with registers/erasable

## Decoder Collision Resolution

The decoder checks instructions in priority order:
1. 6-bit quarter codes (e.g., 010.0 for READ H)
2. 6-bit whole codes (e.g., 013 for DCA)
3. 3-bit basic instructions (e.g., 00. for TC)

However, 3-bit instructions like TC must also validate address ranges:
- TC requires address >= 04000 (fixed memory)
- This prevents false matches on register/erasable addresses
- Allows 6-bit opcodes in range 04000-07777 to take precedence

## References

- GO instruction (00.04000): Restarts program at address 04000
- CPU register map: agcis_3_central_processor.pdf, pages 5-6
- Memory organization: agcis_32_blk2_instructions.pdf
