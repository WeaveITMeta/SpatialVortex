# Terminology Update: Diamond → Flux Matrix

**Date**: October 29, 2025  
**Status**: ✅ Complete  
**Scope**: Renamed all "diamond" references to "flux matrix" throughout codebase

---

## Rationale

**SpatialVortex does NOT use the term "diamond"**. The correct terminology is **"flux matrix"** or **"stored flux matrix"** for high-value moments captured in the Confidence Lake.

---

## Changes Made

### 1. Core Source Files

#### `src/storage/confidence_lake/sqlite_backend.rs`
- ✅ `struct Diamond` → `struct StoredFluxMatrix`
- ✅ `store_diamond()` → `store_flux_matrix()`
- ✅ `retrieve_diamond()` → `retrieve_flux_matrix()`
- ✅ `query_sacred_diamonds()` → `query_sacred_flux_matrices()`
- ✅ Database table: `diamonds` → `flux_matrices`
- ✅ All SQL queries updated
- ✅ Variable names: `let mut diamonds` → `let mut flux_matrices`
- ✅ Struct instantiation: `Diamond {` → `StoredFluxMatrix {`
- ✅ Statistics field: `total_diamonds` → `total_flux_matrices`
- ✅ Test function names updated
- ✅ Comments and documentation updated

#### `src/ai/orchestrator.rs`
- ✅ Method call: `store_diamond()` → `store_flux_matrix()`
- ✅ Comments: "Store diamond" → "Store flux matrix"
- ✅ Comments: "Store voice diamond" → "Store voice flux matrix"

---

### 2. API Documentation

#### `docs/api/API_GAPS_ANALYSIS.md`
- ✅ Endpoint: `/confidence-lake/diamonds` → `/confidence-lake/flux-matrices`
- ✅ Endpoint: `/confidence-lake/diamonds/{id}` → `/confidence-lake/flux-matrices/{id}`
- ✅ Response field: `"diamond_id"` → `"flux_matrix_id"`

#### `docs/api/API_COMPLETION_CHECKLIST.md`
- ✅ Checklist items: `diamonds` endpoints → `flux-matrices` endpoints

---

### 3. Database Schema Changes

**Table Name Change**:
```sql
-- OLD
CREATE TABLE IF NOT EXISTS diamonds (...)

-- NEW  
CREATE TABLE IF NOT EXISTS flux_matrices (...)
```

**Index Names Updated**:
```sql
-- OLD
CREATE INDEX IF NOT EXISTS idx_confidence ON diamonds(...)
CREATE INDEX IF NOT EXISTS idx_sacred ON diamonds(...)
CREATE INDEX IF NOT EXISTS idx_created_at ON diamonds(...)

-- NEW
CREATE INDEX IF NOT EXISTS idx_confidence ON flux_matrices(...)
CREATE INDEX IF NOT EXISTS idx_sacred ON flux_matrices(...)
CREATE INDEX IF NOT EXISTS idx_created_at ON flux_matrices(...)
```

**Query Updates**:
```sql
-- All SELECT, INSERT, DELETE queries updated from 'diamonds' to 'flux_matrices'
SELECT * FROM flux_matrices WHERE ...
INSERT INTO flux_matrices (...)
DELETE FROM flux_matrices WHERE ...
```

---

### 4. API Endpoint Changes

**Confidence Lake Endpoints**:

**Before**:
```yaml
GET    /api/v1/confidence-lake/diamonds
GET    /api/v1/confidence-lake/diamonds/{id}
DELETE /api/v1/confidence-lake/diamonds/{id}
```

**After**:
```yaml
GET    /api/v1/confidence-lake/flux-matrices
GET    /api/v1/confidence-lake/flux-matrices/{id}
DELETE /api/v1/confidence-lake/flux-matrices/{id}
```

---

### 5. Rust Type Changes

**Struct Rename**:
```rust
// OLD
pub struct Diamond {
    pub id: i64,
    pub confidence: f64,
    // ...
}

// NEW
pub struct StoredFluxMatrix {
    pub id: i64,
    pub confidence: f64,
    // ...
}
```

**Method Signatures**:
```rust
// OLD
pub async fn store_diamond(&self, output: &ASIOutput) -> Result<i64>
pub async fn retrieve_diamond(&self, id: i64) -> Result<Diamond>
pub async fn query_sacred_diamonds(&self) -> Result<Vec<Diamond>>

// NEW
pub async fn store_flux_matrix(&self, output: &ASIOutput) -> Result<i64>
pub async fn retrieve_flux_matrix(&self, id: i64) -> Result<StoredFluxMatrix>
pub async fn query_sacred_flux_matrices(&self) -> Result<Vec<StoredFluxMatrix>>
```

**Statistics Struct**:
```rust
// OLD
pub struct LakeStats {
    pub total_diamonds: usize,
    // ...
}

// NEW
pub struct LakeStats {
    pub total_flux_matrices: usize,
    // ...
}
```

---

## Migration Guide

### For Developers

If you have local code that uses the old terminology:

1. **Update struct references**:
   ```rust
   // Replace
   use spatial_vortex::storage::confidence_lake::sqlite_backend::Diamond;
   // With
   use spatial_vortex::storage::confidence_lake::sqlite_backend::StoredFluxMatrix;
   ```

2. **Update method calls**:
   ```rust
   // Replace
   lake.store_diamond(&output).await?;
   lake.retrieve_diamond(id).await?;
   lake.query_sacred_diamonds().await?;
   
   // With
   lake.store_flux_matrix(&output).await?;
   lake.retrieve_flux_matrix(id).await?;
   lake.query_sacred_flux_matrices().await?;
   ```

3. **Update field names**:
   ```rust
   // Replace
   stats.total_diamonds
   
   // With
   stats.total_flux_matrices
   ```

### For API Users

Update your API endpoint URLs:

```bash
# OLD
GET  /api/v1/confidence-lake/diamonds
GET  /api/v1/confidence-lake/diamonds/123
DELETE /api/v1/confidence-lake/diamonds/123

# NEW
GET  /api/v1/confidence-lake/flux-matrices
GET  /api/v1/confidence-lake/flux-matrices/123
DELETE /api/v1/confidence-lake/flux-matrices/123
```

Update JSON response field names:

```json
// OLD
{
  "diamond_id": "diamond123"
}

// NEW
{
  "flux_matrix_id": "fm123"
}
```

---

## Database Migration

If you have an existing Confidence Lake database:

### Option 1: Rename Table (Preserves Data)

```sql
-- SQLite
ALTER TABLE diamonds RENAME TO flux_matrices;

-- Update indexes if needed
DROP INDEX IF EXISTS idx_confidence;
DROP INDEX IF EXISTS idx_sacred;
DROP INDEX IF EXISTS idx_created_at;

CREATE INDEX idx_confidence ON flux_matrices(confidence DESC);
CREATE INDEX idx_sacred ON flux_matrices(is_sacred, flux_position);
CREATE INDEX idx_created_at ON flux_matrices(created_at DESC);
```

### Option 2: Fresh Start (Clean Slate)

```sql
-- Drop old table
DROP TABLE IF EXISTS diamonds;

-- New table will be created automatically on first run
-- by SqliteConfidenceLake::initialize_schema()
```

---

## Testing

All tests updated and passing:

```bash
# Run Confidence Lake tests
cargo test --lib sqlite_backend

# Expected tests:
# ✅ test_sqlite_lake_creation
# ✅ test_store_and_retrieve_flux_matrix
# ✅ test_reject_low_signal
# ✅ test_query_by_signal
# ✅ test_sacred_position_filtering
```

---

## Files Modified

### Source Code (2 files)
1. ✅ `src/storage/confidence_lake/sqlite_backend.rs` - Core implementation
2. ✅ `src/ai/orchestrator.rs` - Method calls

### Documentation (2 files)
3. ✅ `docs/api/API_GAPS_ANALYSIS.md` - API gap analysis
4. ✅ `docs/api/API_COMPLETION_CHECKLIST.md` - Completion checklist

### Created (1 file)
5. ✅ `docs/api/TERMINOLOGY_UPDATE_DIAMOND_TO_FLUX_MATRIX.md` - This document

---

## Terminology Standards

Going forward, use these terms:

### ✅ CORRECT:
- **Flux Matrix** - The core 9-position pattern (1-2-4-8-7-5-1 with 3-6-9 sacred)
- **Stored Flux Matrix** - High-value moment stored in Confidence Lake
- **Flux Matrix Entry** - Individual record in Confidence Lake
- **Flux Matrix Data** - Data associated with stored moments

### ❌ INCORRECT:
- ~~Diamond~~ - Do not use
- ~~Sacred Diamond~~ - Use "Sacred Flux Matrix" or "Sacred Position Flux Matrix"
- ~~Diamond Entry~~ - Use "Flux Matrix Entry"

### Alternative Acceptable Terms:
- **High-Value Moment** - Descriptive term for what gets stored
- **Confidence Entry** - Generic term for Confidence Lake records
- **Lake Entry** - Short form for Confidence Lake records

---

## Impact Assessment

### ✅ Benefits:
1. **Consistent Terminology** - Aligns with core SpatialVortex concepts
2. **Clear Semantics** - "Flux Matrix" clearly indicates the data structure
3. **Reduced Confusion** - No ambiguity about what a "diamond" means
4. **Better Documentation** - Self-explanatory API endpoints
5. **Professional Naming** - More technical and precise

### ⚠️ Breaking Changes:
1. **API Endpoints** - URLs changed (requires client updates)
2. **Rust Types** - Struct and method names changed (requires code updates)
3. **Database Schema** - Table and column names changed (requires migration)
4. **JSON Fields** - Response field names changed (requires parser updates)

### 🔧 Migration Effort:
- **Source Code**: ~30 minutes (find/replace + testing)
- **API Clients**: ~15 minutes per client (endpoint URL updates)
- **Database**: ~5 minutes (run ALTER TABLE or fresh start)
- **Documentation**: ✅ Already complete

---

## Verification Checklist

- [x] All Rust source files updated
- [x] All method names updated
- [x] All struct names updated
- [x] All database queries updated
- [x] All SQL schema updated
- [x] All test function names updated
- [x] All API documentation updated
- [x] All endpoint URLs updated
- [x] All JSON field names updated
- [x] All comments and docstrings updated
- [x] Compilation successful
- [x] Tests passing
- [x] Migration guide provided

---

## Rollout Plan

### Phase 1: Internal (Complete)
- [x] Update source code
- [x] Update documentation
- [x] Run tests
- [x] Commit changes

### Phase 2: Development (Next)
- [ ] Notify development team
- [ ] Update local databases
- [ ] Test API integrations
- [ ] Update any scripts/tools

### Phase 3: Staging (Before Production)
- [ ] Deploy to staging
- [ ] Run migration scripts
- [ ] Test all endpoints
- [ ] Update API clients

### Phase 4: Production (When Ready)
- [ ] Schedule maintenance window
- [ ] Backup database
- [ ] Run migration
- [ ] Deploy updated code
- [ ] Verify all systems

---

## Questions & Answers

**Q: Why not keep backward compatibility?**  
A: The term "diamond" was never in the official terminology and creates confusion. Better to make a clean break now while the system is still in development.

**Q: What about existing data?**  
A: Use the ALTER TABLE migration to preserve all existing data while renaming the table.

**Q: Do I need to update my API clients immediately?**  
A: Yes, the new endpoint URLs are `/confidence-lake/flux-matrices` instead of `/confidence-lake/diamonds`.

**Q: Can I use "diamond" informally?**  
A: No, please use "flux matrix" or "high-value moment" to maintain consistent terminology across the project.

---

## Contact

For questions about this terminology update:
- Review this document
- Check the migration guide above
- Test with the provided examples
- Verify your code compiles with the new names

---

**Status**: ✅ COMPLETE  
**Next Action**: Notify team and begin rollout phases
