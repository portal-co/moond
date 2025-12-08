# ref/ Directory Cleanup Goals

> Created: 2025-12-08T04:04:22.664Z
> 
> This file tracks goals for cleaning up the `ref/` directory structure to make it safer and easier to work on.

## Context

The `ref/` directory contains documentation extracted from AGC PDFs, organized into instruction documentation, CPU documentation, and supporting materials. Current structure has some duplication, orphaned files, and unclear organization that should be addressed.

**Current State:**
- 104 markdown files total
- Mixed organization: some files at root level, others properly nested
- Duplicate/orphaned instruction files (e.g., ref/ccs_k.md vs ref/block1/instr/ccs_k.md)
- Pseudocode stub directory (ref/block2/pseudocode/) with 21 files of uncertain status
- Old processing artifacts (TEMP_INSTR_CHANGES.md)

## Cleanup Goals

### Phase 1: Structural Organization (High Priority) ✅ COMPLETE

- [x] **Audit duplicate instruction files**: Identify all instruction files at ref/ root level that have proper versions in ref/block1/ or ref/block2/. Create a manifest listing:
  - Duplicates (e.g., ref/ccs_k.md vs ref/block1/instr/ccs_k.md)
  - Orphaned files (exist at root but not in proper location)
  - Status recommendation (keep, merge, or archive)
  - **COMPLETED:** See ref/CLEANUP_MANIFEST.md for full audit (2025-12-08T04:16:00)

- [x] **Resolve ref/block2/pseudocode/ directory**: Determine status of 21 pseudocode stub files:
  - Compare with main block2 instruction files
  - Identify if they're older versions, working drafts, or complement main files
  - Decision: merge useful content into main files, archive or delete obsolete stubs
  - Document decision rationale in ref/block2/pseudocode/README.md before action
  - **COMPLETED:** All 21 files archived to ref/_archive/pseudocode_stubs/ (2025-12-08T04:16:00)

- [x] **Clean up root-level files**: Move or consolidate files that should be nested:
  - ref/ccs_k.md, ref/dv_k.md, ref/mp_k.md, ref/su_k.md → evaluate vs block1/instr/ versions
  - Keep only: important_goals.md, cleanup_goals.md, CONVERSATION_SUMMARY.md, TODO_AUDIT.md at root
  - Move general docs (agcis_2_machine_instructions*.md) to ref/sources/ or ref/overview/
  - **COMPLETED:** 4 duplicate instruction files archived, 3 source docs moved to ref/sources/ (2025-12-08T04:16:00)

- [x] **Remove temporary/working files**:
  - ref/TEMP_INSTR_CHANGES.md → review content, extract anything useful to TODO_AUDIT.md, then delete
  - Any other files marked TEMP, WIP, or similar
  - **COMPLETED:** TEMP_INSTR_CHANGES.md archived to ref/_archive/working/ (2025-12-08T04:16:00)

### Phase 2: Documentation Completeness (Medium Priority)

- [ ] **Create directory README files**: Add README.md to each major directory explaining:
  - ref/block1/instr/README.md - Block-1 instruction docs overview
  - ref/block2/README.md - Block-2 documentation structure and differences
  - ref/cpu/README.md - CPU subsystem documentation index
  - ref/definitions/README.md - Canonical type and helper definitions
  - ref/moon/README.md - Source PDF inventory and descriptions

- [ ] **Document file naming conventions**: Create ref/CONVENTIONS.md describing:
  - Instruction file naming (opcode_variant.md format)
  - Pseudocode style requirements (Block-1 vs Block-2)
  - Citation format for PDF references
  - TODO:VERIFY marker usage

- [ ] **Verify all cross-references**: Scan all markdown files for broken links:
  - Links to moved/deleted files
  - References to non-existent sections
  - Update or remove broken references

### Phase 3: Content Validation (Lower Priority)

- [ ] **Audit incomplete instruction files**: List all instruction files that are stubs or incomplete:
  - Files < 500 bytes (likely placeholders)
  - Files missing pseudocode sections
  - Files missing audit/resolution blocks
  - Prioritize for completion based on Block-2 > Block-1 priority

- [ ] **Standardize file headers**: Ensure all instruction files have:
  - Proper source citation (PDF + page numbers)
  - Summary section
  - Modernized pseudocode section
  - Audit/Resolution block (if TODO:VERIFY present)
  - Consistent section ordering

- [ ] **Validate TODO:VERIFY inventory**: Cross-check TODO_AUDIT.md against actual markers:
  - Grep all TODO:VERIFY markers: `grep -r "TODO:VERIFY" ref/ | wc -l`
  - Ensure TODO_AUDIT.md accurately reflects current count and locations
  - Update TODO_AUDIT.md if discrepancies found

### Phase 4: Safety & Quality (Ongoing)

- [ ] **Establish backup protocol**: Before any bulk deletions/moves:
  - Create git tag: `git tag pre-cleanup-YYYYMMDD`
  - Document changes in commit messages with [CLEANUP] prefix
  - Keep deleted content available in git history

- [ ] **Create safety checklist**: Document in ref/CONVENTIONS.md:
  - Never delete files without git tag backup
  - Never merge duplicates without content comparison
  - Always update cross-references after moves
  - Run TODO:VERIFY count before/after major changes

- [ ] **Add validation tests**: Create simple scripts to validate:
  - No broken markdown links (can use markdown-link-check)
  - All instruction files have required sections
  - TODO:VERIFY count matches TODO_AUDIT.md
  - No duplicate filenames across directories

## Success Criteria

The `ref/` directory is considered "clean and safe to work on" when:
1. ✅ No duplicate instruction files exist (one authoritative location per instruction)
2. ✅ All root-level files are either index/summary docs or properly justified
3. ✅ Every subdirectory has a README.md explaining its purpose
4. ✅ All cross-references are valid (no broken links)
5. ✅ Conventions are documented (file naming, citation format, style)
6. ✅ Backup/safety procedures are documented and followed
7. ✅ File status is clear (complete vs stub vs deprecated)

## Migration Path Example

For safety, use this approach when consolidating duplicates:

```bash
# 1. Create backup tag
git tag pre-cleanup-$(date +%Y%m%d)

# 2. Compare files
diff ref/ccs_k.md ref/block1/instr/ccs_k.md

# 3. If root version is older/incomplete, note it
echo "ref/ccs_k.md: older version, superseded by block1/instr/ccs_k.md" >> ref/MIGRATION_LOG.md

# 4. Move to archive instead of deleting
mkdir -p ref/_archive/superseded
git mv ref/ccs_k.md ref/_archive/superseded/

# 5. Commit with clear message
git commit -m "[CLEANUP] Archive superseded ref/ccs_k.md - see block1/instr/ccs_k.md"
```

## Progress Tracking

Track cleanup progress with checkboxes above. When a phase is complete, add timestamp and notes here:

- **Phase 1: ✅ COMPLETE** (2025-12-08T04:16:00.000Z)
  - Archived 4 duplicate root-level instruction files to ref/_archive/superseded/
  - Archived 21 pseudocode stub files to ref/_archive/pseudocode_stubs/
  - Archived TEMP_INSTR_CHANGES.md to ref/_archive/working/
  - Moved 3 source documentation files to ref/sources/
  - Removed empty ref/instr/ directory
  - Created comprehensive CLEANUP_MANIFEST.md audit document
  - Created git tag: pre-cleanup-20251208
  - All changes committed with [CLEANUP] prefix
  - Root level reduced from 12 to 5 files (only index/tracking docs remain)
  
- Phase 2: Not started
- Phase 3: Not started
- Phase 4: Not started

---

Last updated: 2025-12-08T04:16:00.000Z
