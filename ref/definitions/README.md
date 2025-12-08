# Canonical Type and Helper Definitions

> Directory: `ref/definitions/`

## Overview

This directory contains canonical definitions for types, helpers, and common instruction components shared across all AGC instruction documentation. These are the "single source of truth" for standard behaviors.

## Files

### Core Definitions

**`Instruction.md`** - Instruction Type & Structure
- Defines the `Instruction` typedef for 16-bit instruction words
- Describes instruction encoding: order code, address field, parity
- Helper functions: `extract_order_code()`, `extract_address()`
- Referenced by all instruction documentation

**`STD2.md`** - Standard Subinstruction 2 (STD2)
- Canonical definition of the STD2 finalization sequence
- Used by most instructions to complete execution
- Handles: increment Z, stage next instruction, update B/SQ
- Block-1 convention: reference this file, don't inline

**`EXTEND.md`** - Extended Address Mode (EXTEND)
- Defines EXTEND instruction behavior
- How it modifies the next instruction for extended addressing
- Used by NDX and other extended-mode instructions

## Purpose

These files exist to:
1. **Prevent duplication** - Define once, reference everywhere
2. **Ensure consistency** - Single authoritative definition per concept
3. **Simplify updates** - Change in one place propagates to all references
4. **Document standards** - Explicit conventions for all instruction docs

## Usage Conventions

### Block-1 Style
**Use canonical helpers by reference:**
```c
void AD_K(uint16_t K) {
    // ... instruction-specific logic ...
    STD2();  // Canonical helper (see ref/definitions/STD2.md)
}
```

### Block-2 Style
**Inline small helpers with annotation:**
```c
void AD_K(uint16_t K) {
    // ... instruction-specific logic ...
    
    // Inline STD2 for Block-2 timing clarity
    // (See ref/definitions/STD2.md for canonical version)
    Z = Z + 1;
    S = Z;
    // ... inlined STD2 steps ...
}
```

Add "Inline notes" section explaining why inlined (timing, subinstruction fusion).

## File Count

This directory contains **3 canonical definition files**.

## Relationship to Other Documentation

### Referenced By
- All instruction files in `ref/block1/instr/`
- All instruction files in `ref/block2/`
- CPU documentation in `ref/cpu/` (complementary, not redundant)

### Complements
- `ref/cpu/registers.md` - Register and type definitions (hardware view)
- `ref/definitions/` - Instruction components (software/ISA view)

### Distinction
- **CPU docs** describe hardware: registers, adder, parity block
- **Definitions docs** describe ISA abstractions: instruction structure, common sequences

## Canonical Types

From `Instruction.md`:
```c
typedef uint16_t Instruction;  // 16-bit instruction word

// Instruction structure (conceptual):
// Bits 1-3:   Order code (QC field)
// Bits 4-15:  Address field (K)
// Bit 0:      Parity
```

## Canonical Helpers

### Instruction Decoding
```c
uint8_t extract_order_code(Instruction instr);  // Get QC field
uint16_t extract_address(Instruction instr);    // Get K field
```

### Standard Sequences
```c
void STD2();      // Standard completion sequence
void EXTEND();    // Extended address mode setup
```

See individual files for detailed implementations.

## Source Material

These definitions are synthesized from:
- **AGCIS Issue 2** (`ref/moon/agcis_2_machine_instructions.pdf`) - Instruction behavior
- **AGCIS Issue 3** (`ref/moon/agcis_3_central_processor.pdf`) - CPU sequencing
- Extracted patterns common across multiple instructions

## Status and Quality

### Completeness
- Core instruction abstractions documented
- STD2 and EXTEND defined based on AGCIS descriptions
- Instruction type and helpers specified

### Limitations
- Some edge cases may need TODO:VERIFY markers
- Additional helpers may be needed as more instructions are documented
- Block-2 may require additional definitions in `ref/block2/definitions/`

## Future Additions

Potential additional canonical definitions:
- `STMIC.md` - Standard memory inquiry cycle (if needed as explicit helper)
- `PINC_MINC.md` - Standard increment/decrement overflow handling
- `E_memory.md` - E-register handling for extended instructions

Add files here when a pattern appears in 3+ instruction files.

## Usage Guidelines

### When to Reference
✅ Use canonical helpers when:
- Behavior is identical across instructions
- STD2, EXTEND, or other common sequence needed
- Instruction decoding/encoding required

### When to Inline (Block-2)
✅ Inline when:
- Block-2 timing requires fused subinstructions
- Helps clarify instruction-specific control flow
- Always add "Inline notes" explaining rationale

### When to Extend
✅ Add new canonical definitions when:
- Pattern appears in 3+ instruction files
- Behavior is truly identical (not just similar)
- Abstraction reduces complexity without hiding important details

## Related Documentation

- **CPU Documentation:** `ref/cpu/` - Hardware registers and subsystems
- **Block-1 Instructions:** `ref/block1/instr/` - Uses these definitions (by reference)
- **Block-2 Instructions:** `ref/block2/` - Uses these definitions (may inline)
- **Block-2 Definitions:** `ref/block2/definitions/` - Block-2-specific extensions if needed

## Maintenance Notes

When updating canonical definitions:
1. Update the definition file itself
2. Add changelog note in the file
3. Review all instruction files that reference it
4. Update cross-references if behavior changes
5. Commit with `[CANONICAL]` or `[AI]` prefix

---

Last updated: 2025-12-08T04:19:26.568Z
