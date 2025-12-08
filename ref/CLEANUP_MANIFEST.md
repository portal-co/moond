# ref/ Directory Cleanup Manifest

> Created: 2025-12-08T04:11:39.625Z
> 
> This manifest documents all duplicate, orphaned, and temporary files discovered during Phase 1 cleanup audit.

## Phase 1 Audit Results

### Duplicate Instruction Files (Root vs Nested)

**Status: Root versions are OLDER, missing audit blocks and TODO:VERIFY markers**

| Root File | Size | Lines | Nested Version | Size | Lines | Recommendation |
|-----------|------|-------|----------------|------|-------|----------------|
| ref/ccs_k.md | 169B | 3 | ref/block1/instr/ccs_k.md | 5.4K | 93 | ARCHIVE (already a redirect stub) |
| ref/dv_k.md | 4.1K | 90 | ref/block1/instr/dv_k.md | 4.9K | 72 | ARCHIVE (no audit blocks, older version) |
| ref/mp_k.md | 3.7K | 72 | ref/block1/instr/mp_k.md | 4.4K | 59 | ARCHIVE (no audit blocks, older version) |
| ref/su_k.md | 2.5K | 59 | ref/block1/instr/su_k.md | 4.2K | 56 | ARCHIVE (no audit blocks, older version) |

**Key Finding:** Block1/instr/ versions have:
- Audit blocks with PDF citations
- TODO:VERIFY markers (tracked in TODO_AUDIT.md)
- More recent timestamps (2025-12-07T03:37)
- Integration with audit resolution process

**Action:** Archive root versions to ref/_archive/superseded/

### Duplicate Pseudocode Files (block2/pseudocode/ vs block2/)

**Status: 21 pseudocode stub files duplicate main block2 instruction files**

Found duplicates:
- ad_k.md
- dv_e.md
- dv_k.md
- go.md
- linc.md
- minc_c.md
- minc.md
- mp_k.md
- msu_e.md
- ndx_e.md
- pinc.md
- read_h.md
- rpt.md
- shanc.md
- shinc_c.md
- su_k.md
- tc_k.md
- tcsa.md
- write_h.md
- xch_e.md
- xch_k.md

**Sample Comparison:**

```bash
# ref/block2/go.md: 162 lines, complete with audit blocks
# ref/block2/pseudocode/go.md: 479 bytes, minimal stub
```

**Key Finding:** Pseudocode directory contains:
- Older working drafts from pseudocode generation phase
- Much smaller file sizes (most < 1KB)
- Missing audit resolution blocks
- Created earlier (Dec 7 21:44 vs Dec 7 03:37 for main files)

**Action:** Archive entire ref/block2/pseudocode/ directory with explanation

### Temporary/Working Files

| File | Size | Purpose | Recommendation |
|------|------|---------|----------------|
| ref/TEMP_INSTR_CHANGES.md | 57 lines | Tracking file from pseudocode conversion (2025-12-07T08:44:38) | ARCHIVE (historical record, no longer active) |

**Action:** Archive to ref/_archive/working/

### Root-Level Documentation Files

**Files to KEEP at root (index/summary/tracking):**
- ref/important_goals.md ✅ (primary goals tracking)
- ref/cleanup_goals.md ✅ (this cleanup plan)
- ref/CONVERSATION_SUMMARY.md ✅ (project history)
- ref/TODO_AUDIT.md ✅ (central TODO tracking)
- ref/CLEANUP_MANIFEST.md ✅ (this file)

**Files to MOVE/REORGANIZE:**
- ref/agcis_2_machine_instructions.md → ref/sources/ (overview doc)
- ref/agcis_2_machine_instructions_index.md → ref/sources/ (index)
- ref/agcis_2_machine_instructions_instructions.md → ref/sources/ (redirects)

### Empty/Nearly Empty Directories

**ref/instr/** - Empty directory (only .gitkeep-style entry)
- Originally intended for combined Block-1/Block-2 instruction docs
- Convention changed to ref/block1/instr/ and ref/block2/ separation
- **Action:** Remove or document purpose in README.md

## Execution Plan

### Step 1: Create Archive Structure (SAFE - no deletions)

```bash
mkdir -p ref/_archive/superseded
mkdir -p ref/_archive/working
mkdir -p ref/_archive/pseudocode_stubs
mkdir -p ref/sources
```

### Step 2: Archive Root-Level Duplicates

```bash
# Create pre-cleanup tag
git tag pre-cleanup-20251208

# Archive old root-level instruction files
git mv ref/ccs_k.md ref/_archive/superseded/
git mv ref/dv_k.md ref/_archive/superseded/
git mv ref/mp_k.md ref/_archive/superseded/
git mv ref/su_k.md ref/_archive/superseded/

git commit -m "[CLEANUP] Archive superseded root-level instruction files — 2025-12-08

- Moved ccs_k, dv_k, mp_k, su_k to _archive/superseded/
- Block1/instr/ versions are authoritative (have audit blocks)
- See ref/CLEANUP_MANIFEST.md for details"
```

### Step 3: Archive Pseudocode Stubs

```bash
git mv ref/block2/pseudocode ref/_archive/pseudocode_stubs/

git commit -m "[CLEANUP] Archive Block-2 pseudocode stubs — 2025-12-08

- Moved entire ref/block2/pseudocode/ directory to _archive/
- Main ref/block2/*.md files are authoritative and complete
- Stubs were working drafts from initial conversion phase
- See ref/CLEANUP_MANIFEST.md for details"
```

### Step 4: Archive Temporary Files

```bash
git mv ref/TEMP_INSTR_CHANGES.md ref/_archive/working/

git commit -m "[CLEANUP] Archive temporary tracking file — 2025-12-08

- TEMP_INSTR_CHANGES.md was a working file from Dec 7 conversion
- No longer actively used; preserved for historical reference
- See ref/CLEANUP_MANIFEST.md for details"
```

### Step 5: Reorganize Source Documentation

```bash
mkdir -p ref/sources
git mv ref/agcis_2_machine_instructions.md ref/sources/
git mv ref/agcis_2_machine_instructions_index.md ref/sources/
git mv ref/agcis_2_machine_instructions_instructions.md ref/sources/

git commit -m "[CLEANUP] Reorganize source documentation — 2025-12-08

- Moved agcis_2_machine_instructions*.md to ref/sources/
- Keeps root level clean for index/tracking docs only
- See ref/CLEANUP_MANIFEST.md for details"
```

### Step 6: Handle Empty ref/instr/ Directory

```bash
# Document the directory purpose or remove it
rmdir ref/instr/  # If truly empty

git commit -m "[CLEANUP] Remove empty ref/instr/ directory — 2025-12-08

- Directory was from earlier organizational scheme
- Convention now uses ref/block1/instr/ and ref/block2/ separation
- See ref/CLEANUP_MANIFEST.md for details"
```

## Verification Checklist

After each step:
- [ ] Run `git status` to verify only intended files modified
- [ ] Check no unintended files deleted
- [ ] Verify commits have [CLEANUP] prefix and timestamp
- [ ] Test that no markdown links are broken (future Phase 2 task)
- [ ] Confirm backup tag exists: `git tag -l pre-cleanup-*`

## Summary Statistics

**Before Cleanup:**
- Total markdown files: 104
- Root-level instruction files: 4 duplicates
- Pseudocode stub files: 21 duplicates
- Temporary files: 1
- Root-level doc files: 7 (3 to relocate)

**After Cleanup (Projected):**
- Total markdown files: 104 (unchanged - moved to _archive, not deleted)
- Root-level instruction files: 0 duplicates
- Pseudocode stub files: 0 (archived)
- Temporary files: 0 (archived)
- Root-level doc files: 5 (only index/tracking/goals)
- New directories: ref/_archive/, ref/sources/

**Safety:** All files preserved in git history and _archive/ directory. No data loss.

---

Last updated: 2025-12-08T04:11:39.625Z
