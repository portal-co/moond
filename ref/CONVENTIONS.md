# Documentation Conventions

> Created: 2025-12-08T04:20:00.000Z
> 
> This document describes file naming conventions, pseudocode style requirements, citation formats, and TODO:VERIFY marker usage for the AGC documentation project.

## File Naming Conventions

### Instruction Files

**Format:** `<opcode>_<variant>.md`

**Components:**
- `<opcode>`: Instruction mnemonic in lowercase (e.g., `ccs`, `dv`, `mp`, `tc`, `ad`)
- `<variant>`: Address mode or instruction variant indicator

**Variant Codes:**
- `k` = K-type (direct address, basic memory access)
- `e` = E-type (extended addressing, E-memory)
- `c` = C-type (counter/peripheral instruction)
- `f` = F-type (flow control, fixed address)
- `h` = H-type (I/O and hardware interface)

**Examples:**
```
ccs_k.md    # Count, Compare, Skip (K-type)
dv_e.md     # Divide (E-type extended)
pinc_c.md   # Plus Increment (counter type)
tcf_f.md    # Transfer Control to Fixed (flow control)
read_h.md   # Read (I/O hardware type)
```

**Special Cases:**
```
pinc.md     # When instruction has no variants (no suffix)
go.md       # Single-variant instructions
rupt.md     # Interrupt handling (special case)
```

### Directory Organization

**Block-1:**
- Location: `ref/block1/instr/`
- Contains Block-1 specific instruction documentation

**Block-2:**
- Location: `ref/block2/`
- Contains Block-2 specific instruction documentation
- May have different behavior than Block-1 versions

**CPU Subsystems:**
- Location: `ref/cpu/`
- One file per subsystem: `registers.md`, `adder.md`, `parity_block.md`, etc.

**Canonical Definitions:**
- Location: `ref/definitions/`
- Shared types and helpers: `Instruction.md`, `STD2.md`, `EXTEND.md`

## Pseudocode Style

### Modern Unified Style (Both Blocks)

Use modern, readable pseudocode with inline subinstructions and comments referencing canonical definitions:

```c
void AD_K(uint16_t K) {
    // Fetch next instruction (subinstruction STD2 part 1)
    // See ref/definitions/STD2.md for canonical subinstruction definition
    uint16_t next_addr = Z;
    S = next_addr;
    
    // Read operand from memory
    uint16_t operand = memory[K];
    
    // Perform addition
    A = A + operand;
    
    // Branch to next instruction (subinstruction STD2 part 2)
    // See ref/definitions/STD2.md
    Z = Z + 1;
    
    // Decode next instruction
    B = memory[Z];
}
```

**Key Principles:**
- **Inline subinstructions** - Don't call `STD2()` or `STMIC_stage()` as functions
- **Add reference comments** - Point to canonical definition files
- **Use modern terminology** - "branch", "decode", "read", "write" instead of pulses
- **Focus on behavior** - What happens, not how hardware implements it
- **Keep it verifiable** - Code should match instruction semantics
- **Keep it usable** - Implementer can follow without hardware details

**Subinstruction Handling:**
```c
// Example: Inlining STD2 with reference
// Standard completion sequence (STD2)
// See ref/definitions/STD2.md for canonical definition
Z = Z + 1;              // Increment program counter
B = memory[Z];          // Fetch next instruction
SQ = extract_order_code(B);  // Decode operation
```

**Don't:**
❌ Use function calls for subinstructions: `STD2();`
❌ Use hardware pulse names: `RZ`, `WS`, `WG1G`
❌ Separate Block-1 and Block-2 styles
❌ Omit reference comments to canonical definitions

**Do:**
✅ Inline all subinstructions with clear comments
✅ Reference canonical definitions: `// See ref/definitions/STD2.md`
✅ Use modern terms: branch, decode, fetch, store
✅ Show actual behavior in clear, verifiable steps
✅ Use same style for both Block-1 and Block-2

## Modern Terminology

Use clear, modern terms instead of hardware implementation details:

**Prefer Modern Terms:**
- ✅ `branch` - Control flow change (not "transfer control", "TC")
- ✅ `decode` - Extract opcode/address from instruction
- ✅ `fetch` - Read from memory
- ✅ `store` - Write to memory
- ✅ `read` - Get value from register/memory
- ✅ `write` - Set value in register/memory
- ✅ `test` - Check condition (CCS = "count, compare, skip")
- ✅ `skip` - Conditional branch
- ✅ `accumulator` - Register A
- ✅ `program counter` - Register Z

**Avoid Hardware Terms in Pseudocode:**
- ❌ Pulse names: `RZ`, `WS`, `WG1G`, `RL`, `WA`
- ❌ "Stage" (use "fetch" or "read")
- ❌ "Inquiry cycle" (just show the memory read)
- ❌ "Write amplifier" (just show the write operation)
- ❌ "STMIC" (inline the actual fetch operations)

**Example - Before (hardware-focused):**
```c
// STMIC stage
RZ; WS; WY;  // Control pulses
if (S >= 0o20) { RL; }
```

**Example - After (behavior-focused):**
```c
// Fetch next instruction address
uint16_t next_addr = Z;
// Read from memory if address is valid
if (next_addr >= 0o20) {
    instruction = memory[next_addr];
}
```

## Type Conventions

### Canonical Types

Defined in `ref/cpu/registers.md`:

```c
typedef int16_t  int15_t;   // signed 15-bit value (in 16-bit container)
typedef uint16_t uint15_t;  // unsigned 15-bit value (low 15 bits used)
typedef uint16_t Instruction;  // 16-bit instruction word
```

### Type Usage

**In code:**
```c
uint16_t word;      // Full 16-bit AGC word (including parity)
int16_t signed_val; // Signed 16-bit value
uint15_t addr;      // 15-bit address or value field
```

**In prose:**
- Use `(u)int15_t` when discussing 15-bit value fields
- Use `uint16_t`/`int16_t` for full words
- Use `uint12_t` for 12-bit addresses (program counter, S register)
- Use `uint8_t` for opcodes (4-bit) and flags
- Note that AGC words have: Bit 0=parity, Bits 1-15=value (some contexts show bit 16 as sign)

### Helper Functions

Simple utility functions are acceptable for common operations:

```c
// Type conversion helpers
int32_t sign_extend15(uint15_t v);  // Sign extend 15-bit to 32-bit
uint8_t extract_order_code(uint16_t instr);  // Get opcode from instruction
uint12_t extract_address(uint16_t instr);   // Get address field

// Bit manipulation
uint8_t count_bits(uint16_t v);  // Count set bits (for parity)
```

**Prefer inline code over abstraction** - When in doubt, show the actual operations rather than hiding them in function calls.

See `ref/cpu/registers.md` for canonical type definitions.

## Octal Notation

**Use `0o` prefix for octal constants, especially for values that are multiples of 3 bits wide:**

```c
✅ if (S >= 0o20) { ... }      // Address comparison
✅ Z = 0o4000;                  // 12-bit address (4 octal digits)
✅ mask = 0o7777;               // 15-bit mask (5 octal digits)
✅ opcode = 0o7;                // 3-bit opcode
✅ channel = 0o177;             // 7-bit I/O channel

❌ if (S >= 020) { ... }       // Don't use C-style octal
❌ Z = 4000;                   // Ambiguous: decimal or octal?
```

**Why:**
- The `0o` prefix is unambiguous (matches Python 3 style)
- AGC architecture is naturally octal (3-bit groups)
- Addresses, masks, and bit fields are clearer in octal
- 15-bit values fit perfectly in 5 octal digits (0o00000 - 0o77777)

**Prefer octal for:**
- Memory addresses (12-bit: 0o0000 - 0o7777)
- Bit masks (3, 6, 9, 12, 15-bit values)
- Opcodes and instruction fields
- I/O channel numbers

**Use decimal/hex for:**
- Counts and loop indices
- Bit positions (0-14)
- When decimal is more natural (e.g., "add 1", "divide by 16")

## Citation Format

### Source PDFs

**Format at top of file:**
```markdown
Source: `<pdf_filename>` — pages X–Y (figs. A–B; tables C–D).
```

**Examples:**
```markdown
Source: `agcis_2_machine_instructions.pdf` — pages 36–45 (figs. 2-12..2-16; table 2-3).
Source: `agcis_3_central_processor.pdf` — pages 6–12 (table 3-1; fig. 3-2).
Source: `agcis_32_blk2_instructions.pdf` — pages 120–135.
```

### In-Text References

**Cross-referencing other files:**
```markdown
See `ref/cpu/registers.md` for type definitions.
Overflow handling per `ref/cpu/adder.md`.
Canonical STD2 defined in `ref/definitions/STD2.md`.
```

**Referencing audit tracking:**
```markdown
See `ref/TODO_AUDIT.md` for centralized tracking.
Tracked in TODO_AUDIT.md (entry added 2025-12-07T08:13:47.632Z).
```

### Audit Blocks

**Standard audit block format:**
```markdown
Audit
- Scanned repository PDFs (ref/moon/AEAProgrammingReference.pdf, ref/moon/agcis_3_central_processor.pdf, ref/moon/agcis_2_machine_instructions.pdf) on 2025-12-07 for authoritative support.
- Action: retain `TODO:VERIFY` marker and consult ref/TODO_AUDIT.md for central tracking.

Audit resolution (YYYY-MM-DDTHH:MM:SS.sssZ):
- Reviewed AGCIS Issue 2 pages X–Y; behavior supported by figures/tables.
- Status: [resolved/partially resolved/TODO:VERIFY retained]
```

## TODO:VERIFY Marker Usage

### When to Use

Mark behaviors that need verification with `TODO:VERIFY`:

```markdown
✅ TODO:VERIFY (OCR unreadable in source PDF)
✅ TODO:VERIFY (inferred from training/model, no PDF support found)
✅ TODO:VERIFY (ambiguous: could be behavior A or B)
✅ TODO:VERIFY (hardware timing detail not documented in PDFs)
```

### Format

**In prose:**
```markdown
The overflow bit behavior is TODO:VERIFY (ambiguous in AGCIS Issue 2 pages 46-50).
```

**In code comments:**
```c
// TODO:VERIFY: Does overflow set PINC/MINC on this operation?
if (overflow) {
    // Uncertain behavior
}
```

### Rationale Requirements

**Always include rationale after TODO:VERIFY:**

```markdown
❌ TODO:VERIFY
✅ TODO:VERIFY (OCR unclear)
✅ TODO:VERIFY (no PDF support; inferred from model)
✅ TODO:VERIFY (ambiguous between two interpretations)
```

**Common rationales:**
- `(OCR unreadable)` - Source PDF text is illegible
- `(inferred from training/model)` - No explicit PDF documentation found
- `(ambiguous)` - Multiple valid interpretations possible
- `(no hardware timing details)` - Timing not specified in PDFs
- `(E-memory restore unclear)` - Block-2 E-memory edge case
- `(SCALER width uncertain)` - Channel width alignment question

### Tracking

All TODO:VERIFY markers are tracked in `ref/TODO_AUDIT.md`:
- Initial discovery recorded with file location
- Audit status tracked (supported/ambiguous/pending)
- Resolution attempts documented
- Central reference for all unresolved items

## File Structure Standards

### Instruction Files

**Standard sections (in order):**

```markdown
# <INSTRUCTION> — <Description> (modernized) [or (Block-2)]

Source: `<pdf>` — pages X–Y (figs., tables).

Summary
- Operation: Brief description
- Key behaviors and characteristics

[Representation notes] (if needed for type/encoding clarifications)

Micro-op (C-like pseudocode) [or: Modernized pseudocode]
<code block>

[Inline notes] (for Block-2 if helpers were inlined)

Notes
- Edge cases
- Special behaviors
- Cross-references

[TODO:VERIFY items] (if applicable)

Audit
<audit blocks with timestamps>
```

### Required Sections
- ✅ Header with instruction name and description
- ✅ Source citation (PDF + pages)
- ✅ Summary section
- ✅ Pseudocode section (with inline subinstructions and reference comments)
- ✅ Audit block (if file has TODO:VERIFY markers)

### Optional Sections
- Representation notes (for encoding clarifications)
- Extended notes (for complex behaviors)
- Block differences (note if Block-1 vs Block-2 behavior differs)

### Deprecated Sections
- ❌ "Inline notes" - No longer needed with unified style
- ❌ Separate "Micro-op" sections - Use "Pseudocode" consistently

## Commit Message Conventions

### Format

```
[PREFIX] Brief description — YYYY-MM-DDTHH:MM:SS.sssZ

- Bullet point detail 1
- Bullet point detail 2
- Reference to documentation if applicable
```

### Prefixes

- `[AI]` - AI-generated commits (general)
- `[CLEANUP]` - Directory/file organization changes
- `[CANONICAL]` - Changes to canonical definitions
- `[AUDIT]` - Audit resolution or TODO:VERIFY updates
- `[DOC]` - Documentation improvements (non-code)

### Examples

```
[AI] Add Block-2 CCS_E instruction with audit notes — 2025-12-07T08:25:31.234Z

- Created ref/block2/ccs_e.md with inline pseudocode
- Added TODO:VERIFY marker for plus/minus-zero encoding
- Referenced AGCIS Issue 32 pages 45-52
```

```
[CLEANUP] Archive superseded root-level instruction files — 2025-12-08T04:13:00.000Z

- Moved ccs_k, dv_k, mp_k, su_k to _archive/superseded/
- Block1/instr/ versions are authoritative (have audit blocks)
- See ref/CLEANUP_MANIFEST.md for details
```

## Safety & Quality Checklist

### Before Bulk Changes

- [ ] Create git tag: `git tag pre-<change>-YYYYMMDD`
- [ ] Document intended changes in manifest or plan
- [ ] Verify no unintended files in scope
- [ ] Run TODO:VERIFY count before: `grep -r "TODO:VERIFY" ref/ | wc -l`

### After Bulk Changes

- [ ] Run `git status` and verify only intended files modified
- [ ] Run TODO:VERIFY count after and compare
- [ ] Verify no broken markdown links (future: automate with tool)
- [ ] Commit with appropriate prefix and timestamp
- [ ] Update tracking documents (goals, audit, etc.)

### Never Do

- ❌ Delete files without git tag backup
- ❌ Merge duplicates without content comparison
- ❌ Change file names without updating cross-references
- ❌ Add TODO:VERIFY without rationale
- ❌ Remove TODO:VERIFY without audit resolution

## Validation Commands

```bash
# Count markdown files
find ref -name "*.md" | wc -l

# Count TODO:VERIFY markers
grep -r "TODO:VERIFY" ref/ | wc -l

# List files missing source citations
grep -L "^Source:" ref/block1/instr/*.md ref/block2/*.md

# Find TODO:VERIFY without rationale (should be empty)
grep "TODO:VERIFY$" ref/ -r

# Check for old-style octal (0 prefix) - should find none in new code
grep -E "\b0[0-7]{3,}\b" ref/block1/instr/*.md ref/block2/*.md | grep -v "^#" | grep -v "0o"
```

## Directory Standards

### Every Major Directory Must Have

- ✅ `README.md` - Explains purpose, contents, conventions
- ✅ Clear naming convention for files
- ✅ Documented relationship to other directories

### README.md Structure

```markdown
# <Directory Name>

> Directory: `ref/<path>/`
> Source: <if applicable>

## Overview
[Brief description]

## Files
[List and describe files]

## File Count
[Number of files]

## Purpose
[Why this directory exists]

## Related Documentation
[Links to other directories/files]

---

Last updated: YYYY-MM-DDTHH:MM:SS.sssZ
```

---

Last updated: 2025-12-08T04:20:00.000Z
