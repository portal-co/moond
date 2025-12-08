# Phase 3: Content Validation Audit

> Started: 2025-12-08T04:38:00.000Z
> Status: IN PROGRESS (with modernization refactor)

## Overview

Phase 3 involves validating content completeness, standardizing file headers, and validating the TODO:VERIFY inventory. This phase is being performed concurrently with a **modernization refactor** that unifies Block-1 and Block-2 pseudocode styles.

## Modernization Refactor (In Progress)

### Goal
Convert all instruction documentation to use unified, modern pseudocode style:
- Inline subinstructions with reference comments (not function calls)
- Use modern terminology (branch, fetch, store, decode)
- Remove hardware pulse names (RZ, WS, WG1G, etc.)
- Focus on verifiable behavior, not hardware implementation
- Apply same style to both Block-1 and Block-2

### Conventions Updated
✅ `ref/CONVENTIONS.md` modernized (2025-12-08T04:37:00)
- Removed separate Block-1/Block-2 style sections
- Added "Modern Terminology" section
- Updated all pseudocode examples
- Clarified octal usage for 3-bit-multiple values

## Phase 3 Tasks

### Task 1: Audit Incomplete Instruction Files

**Criteria for "incomplete":**
- Files < 500 bytes (likely placeholders)
- Files missing pseudocode/micro-op section
- Files missing summary section
- Files with placeholder content ("TODO", "WIP")

**Initial Audit Results:**

Files < 500 bytes: **0** ✅
- All instruction files have substantial content

Files missing pseudocode section: **1**
- `ref/block2/differences.md` - Not an instruction file, expected

Files missing source citation: **TBD** (checking...)

**Conclusion:** No stub files found. All instruction files have content and structure.

### Task 2: Standardize File Headers

**Required sections per CONVENTIONS.md:**
- Header with instruction name and description
- Source citation (PDF + pages)
- Summary section
- Pseudocode section
- Audit block (if TODO:VERIFY present)

**Modernization refactor adds:**
- Inline subinstructions (not function calls)
- Reference comments to canonical definitions
- Modern terminology (branch, fetch, store)
- Unified style for Block-1 and Block-2

**Audit approach:**
1. Sample files from each directory
2. Check header compliance
3. Identify files needing modernization
4. Apply modern style during Phase 3 validation

### Task 3: Validate TODO:VERIFY Inventory

**Current TODO:VERIFY count:**
- Total in instruction/CPU files: **~70-76** (per previous audits)
- Tracked in `ref/TODO_AUDIT.md`

**Validation steps:**
1. Count actual TODO:VERIFY markers: `grep -r "TODO:VERIFY" ref/`
2. Compare with TODO_AUDIT.md entries
3. Verify each has rationale (OCR/inferred/ambiguous/timing)
4. Update TODO_AUDIT.md if discrepancies found

**Initial count (2025-12-08T04:38:00):**
- Checking...

## Modernization Strategy

### Files to Modernize

**Priority 1: Sample files (examples)**
- Pick 2-3 representative files from Block-1
- Pick 2-3 representative files from Block-2
- Modernize as templates for future work

**Priority 2: Files with hardware pulse references**
- Search for: `RZ`, `WS`, `WG`, `RL`, `STMIC`
- Replace with inline modern code

**Priority 3: Files calling subinstructions as functions**
- Search for: `STD2()`, `STMIC_stage()`, `EXTEND()`
- Inline with reference comments

**Priority 4: Systematic update**
- After templates established, apply to remaining files
- This is ongoing work beyond Phase 3 scope

### Modern Pseudocode Template

```c
void <INSTRUCTION>_<VARIANT>(uint16_t <PARAM>) {
    // [Brief description of what happens]
    
    // Read/fetch operations
    uint16_t operand = memory[address];
    
    // Core instruction logic
    A = A + operand;  // Modern, clear operation
    
    // Condition handling
    if (overflow) {
        // Handle overflow
    }
    
    // Branch to next instruction (STD2 completion)
    // See ref/definitions/STD2.md for canonical subinstruction
    Z = Z + 1;                      // Increment program counter
    uint16_t next = memory[Z];      // Fetch next instruction  
    SQ = extract_order_code(next);  // Decode operation
}
```

## Sample Files for Modernization

### Block-1 Samples (will modernize)
1. `ref/block1/instr/ad_k.md` - Simple arithmetic
2. `ref/block1/instr/ccs_k.md` - Conditional branch
3. `ref/block1/instr/tc_k.md` - Branch/call

### Block-2 Samples (will modernize)
1. `ref/block2/ad_k.md` - Compare with Block-1 version
2. `ref/block2/ccs_e.md` - Extended addressing
3. `ref/block2/read_h.md` - I/O instruction

### Modernization Checklist (per file)

For each file being modernized:
- [ ] Remove hardware pulse names (RZ, WS, WG, etc.)
- [ ] Inline STD2/STMIC subinstructions
- [ ] Add reference comments: `// See ref/definitions/STD2.md`
- [ ] Use modern terms: branch, fetch, store, decode
- [ ] Use octal for 3-bit-multiple values
- [ ] Ensure pseudocode is verifiable and usable
- [ ] Update section headers if needed ("Pseudocode" not "Micro-op")
- [ ] Verify source citation present
- [ ] Verify summary present
- [ ] Check audit block if TODO:VERIFY present

## Phase 3 Deliverables

### In Scope (Phase 3)
✅ Audit incomplete files (completed - no stubs found)
✅ Standardize headers (checking compliance)
✅ Validate TODO:VERIFY inventory (in progress)
✅ Create modernization templates (2-3 sample files per block)
✅ Document modernization approach

### Out of Scope (Future Work)
- Complete modernization of all 62 instruction files
- This is ongoing work after Phase 3 completes
- Templates and conventions established in Phase 3
- Systematic application happens incrementally

## Progress Tracking

**Phase 3 Status:** IN PROGRESS
- ✅ Conventions modernized
- ✅ Stub file audit complete (no stubs found)
- ✅ Template modernization complete (3 templates created)
- ⏳ Header standardization in progress
- ⏳ TODO:VERIFY validation in progress
- ⏳ Citation additions for Block-2 files (15+ files need citations)

**Modernization Refactor Status:** TEMPLATES COMPLETE
- ✅ Conventions documented (ref/CONVENTIONS.md)
- ✅ Template files created (3 modernized examples):
  - `ref/block1/instr/ad_k_MODERN.md` - Simple arithmetic (Block-1)
  - `ref/block1/instr/ccs_k_MODERN.md` - Conditional branch (Block-1)
  - `ref/block2/ad_k_MODERN.md` - Block-2 with PDF citation
- ⏳ Systematic application (ongoing future work, beyond Phase 3 scope)

**PDF Citations Added:**
- ✅ AGCIS Issue 2 pages 36-45 (CCS K instruction)
- ✅ AGCIS Issue 32 pages 92-93 (AD K Block-2 instruction)
- ⏳ 15+ Block-2 files still need citations (ads_e, aug_e, bzf_f, ca_k, etc.)

---

Last updated: 2025-12-08T04:48:00.000Z
