# 📁 Documentation File Governance
**Purpose**: Prevent documentation sprawl and redundancy  
**Rule**: One file per purpose, delete when obsolete  
**Owner**: Project lead

---

## 🚨 The Problem We Had

**Oct 26, 2025 Session**: Created 17 new files in attempt to document status
- 9 files about "consolidation" 
- 3 files about "status"
- 5 other summary files

**Result**: Made the problem worse, not better!

---

## ✅ New Rules (Enforced)

### Rule 1: Single Source of Truth Per Topic

**Status/Progress** → `docs/PROJECT_STATUS.md` ONLY
- Do NOT create: COMPLETION_SUMMARY.md, ORDER_RESTORED.md, PROGRESS_REPORT.md, etc.
- UPDATE the existing file instead

**Critical Path** → `docs/ACTION_PLAN_CRITICAL_PATH.md` ONLY
- Do NOT create: TODO.md, NEXT_STEPS.md, PRIORITIES.md, etc.
- UPDATE the existing file instead

**Gap Analysis** → `docs/IMPLEMENTATION_GAP_REPORT.md` ONLY
- Do NOT create: GAPS.md, MISSING_FEATURES.md, INCOMPLETE.md, etc.
- UPDATE the existing file instead

**Component Details** → `docs/IMPLEMENTATION_STATUS.md` ONLY
- Do NOT create: COMPONENT_STATUS.md, FEATURE_STATUS.md, etc.
- UPDATE the existing file instead

---

### Rule 2: Before Creating, Ask

**Question 1**: Does a file for this purpose already exist?
- Yes → Update it
- No → Proceed to Question 2

**Question 2**: Will this file become redundant in 1 week?
- Yes → Don't create it, update existing
- No → Proceed to Question 3

**Question 3**: Can this be a section in an existing file?
- Yes → Add section to existing file
- No → Create new file BUT add to governance list

---

### Rule 3: Delete When Obsolete

**Completed milestones** → Move to `docs/archive/YYYY-MM/`
- Example: `CHECKPOINT_1_COMPLETE.md` → `archive/2025-10/`

**Superseded docs** → Delete immediately
- Example: Created `PROJECT_STATUS.md` → Delete 6 old status files

**Temporary planning** → Delete after execution
- Example: `CONSOLIDATION_PLAN.md` → Delete after consolidation done

---

### Rule 4: Use Existing Structure

**Status/Progress**:
- `docs/PROJECT_STATUS.md` - Quick overview
- `docs/IMPLEMENTATION_STATUS.md` - Component details
- `docs/IMPLEMENTATION_GAP_REPORT.md` - Gap analysis

**Planning**:
- `docs/ACTION_PLAN_CRITICAL_PATH.md` - Next 20 tasks
- `docs/design/MASTER_ROADMAP.md` - 18-month plan
- `docs/design/FULL_ASI_ROADMAP.md` - 8-week focused plan

**Architecture**:
- `docs/architecture/` - System design
- `docs/research/` - Mathematical foundations
- `docs/specs/` - Technical specs

**Learning**:
- `docs/minimal/UNIFIED_ARCHITECTURAL_FRAMEWORK.md` - Complete architecture
- `docs/MASTER_INDEX.md` - Find any document
- `docs/guides/` - How-to guides

---

## 📊 Approved File List

### Root Status Files (3 ONLY)
1. ✅ `docs/PROJECT_STATUS.md` - Quick overview
2. ✅ `docs/IMPLEMENTATION_STATUS.md` - Component details  
3. ✅ `docs/IMPLEMENTATION_GAP_REPORT.md` - Gap analysis

**Any other status file = REDUNDANT**

### Planning Files (3 ONLY)
1. ✅ `docs/ACTION_PLAN_CRITICAL_PATH.md` - Next 20 tasks
2. ✅ `docs/design/MASTER_ROADMAP.md` - 18-month plan
3. ✅ `docs/design/FULL_ASI_ROADMAP.md` - 8-week plan

**Any other planning file = REDUNDANT**

### Navigation Files (2 ONLY)
1. ✅ `docs/MASTER_INDEX.md` - Complete index
2. ✅ `docs/minimal/README.md` - Getting started

**Any other index file = REDUNDANT**

---

## 🗑️ Deleted Today (Redundant)

**Consolidation spam** (6 files):
- ❌ CONSOLIDATION_ANALYSIS.md
- ❌ CONSOLIDATION_COMPLETE.md
- ❌ CONSOLIDATION_PLAN.md
- ❌ CONSOLIDATION_SUMMARY.md
- ❌ OPTION_D_COMPLETE.md
- ❌ CHECKBOX_UPDATE_SUMMARY.md
- ❌ TASK_STATUS_UPDATE.md

**Replaced by**: `PROJECT_STATUS.md` (single source of truth)

---

## ✅ Checkpoint Process

### When You Complete a Milestone

**DON'T**: Create `MILESTONE_X_COMPLETE.md`

**DO**:
1. Update `docs/PROJECT_STATUS.md` (add to achievements)
2. Update `docs/IMPLEMENTATION_STATUS.md` (mark component complete)
3. Create entry in `docs/milestones/` IF it's major (like Checkpoints 1-3)
4. Update roadmap checkboxes if applicable

---

### When You Start Planning

**DON'T**: Create `NEW_PLAN.md`, `PRIORITIES.md`, `TODO.md`

**DO**:
1. Update `docs/ACTION_PLAN_CRITICAL_PATH.md` (adjust 20 tasks)
2. Update `docs/design/MASTER_ROADMAP.md` (if long-term change)
3. Create milestone-specific plan in `docs/milestones/` ONLY if major initiative

---

### When You Find a Gap

**DON'T**: Create `GAPS.md`, `MISSING.md`, `INCOMPLETE.md`

**DO**:
1. Update `docs/IMPLEMENTATION_GAP_REPORT.md` (component status)
2. Update `docs/PROJECT_STATUS.md` (overall percentage)
3. Add to critical path if blocking

---

## 🎯 File Lifecycle

### Creation
1. Check if file for this purpose exists
2. If not, justify why new file is needed
3. Add to approved list in this document
4. Create with clear purpose statement

### Maintenance
1. Weekly: Review and update status files
2. Monthly: Archive completed milestones
3. Quarterly: Delete obsolete planning docs

### Deletion
1. Completed milestone → Archive to `docs/archive/YYYY-MM/`
2. Superseded doc → Delete immediately
3. Temporary planning → Delete after execution
4. Redundant doc → Delete immediately

---

## 🚫 Common Mistakes (Don't Do This)

### Mistake 1: Session Summary Spam
**Don't**: Create `SESSION_YYYY_MM_DD.md` for every work session
**Do**: Update `PROJECT_STATUS.md` with achievements

### Mistake 2: Multiple Status Files
**Don't**: Create `STATUS_v2.md`, `STATUS_LATEST.md`, `REAL_STATUS.md`
**Do**: Update the ONE status file

### Mistake 3: Consolidation Irony
**Don't**: Create 9 files about consolidation
**Do**: Actually consolidate and delete

### Mistake 4: Checkpoint Spam
**Don't**: Create 100 checkpoint files for minor tasks
**Do**: Create checkpoints only for MAJOR milestones (like Checkpoints 1-3)

### Mistake 5: Planning Paralysis
**Don't**: Create endless planning docs instead of coding
**Do**: Update the 20 critical tasks, then build

---

## 📝 Template: When to Create New File

**Before creating ANY new documentation file, answer**:

1. **Purpose**: What unique purpose does this serve?
2. **Existing**: Does a file for this already exist?
3. **Lifetime**: Will this be obsolete in <1 week?
4. **Alternative**: Can this be a section in existing file?
5. **Approval**: Is this on the approved file list?

**If you answer**:
- Existing = Yes → UPDATE that file
- Lifetime = <1 week → DON'T create
- Alternative = Yes → ADD section to existing
- Approval = No → JUSTIFY or DON'T create

---

## ✅ Success Metrics

**Before governance** (Oct 26 morning):
- 70 markdown files
- 10+ redundant files
- 3 conflicting indexes
- Confusing navigation

**After governance** (Oct 26 evening):
- 63 markdown files (7 deleted)
- 0 redundant files
- 1 master index
- Clear governance

**Target** (ongoing):
- Maintain ~60 files
- Zero redundancy
- Clear purpose per file
- Easy maintenance

---

## 🔄 Review Cadence

**Weekly** (every Friday):
- Review newly created files
- Delete any redundant files
- Update status files

**Monthly** (first Monday):
- Archive completed milestones
- Delete obsolete planning docs
- Review file count

**Quarterly** (first of quarter):
- Major governance review
- Update approved file list
- Clean up accumulated cruft

---

## 🎓 Key Principle

> **"One file per purpose. Update existing before creating new. Delete when obsolete."**

This is not optional. This is how we prevent documentation sprawl.

---

**Enforced by**: Project lead (you!)  
**Review**: Every PR should check for redundant docs  
**Penalty**: Redundant files will be deleted without warning  
**Updated**: 2025-10-26

---

**Status**: Governance established ✅  
**Compliance**: Mandatory 🔒  
**Next Review**: 2025-11-02
