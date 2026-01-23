# 🐛 Bug Fix: Session Store Reactivity Error

**Date**: November 3, 2025  
**Severity**: Critical  
**Status**: ✅ Fixed  
**File**: `web/src/lib/stores/sessionStore.ts`

---

## 🔴 Issue

**Error:**
```javascript
Uncaught (in promise) RangeError: Invalid array length
    at Array.push (<anonymous>)
    at set2 (chunk-E6UGLBRP.js:3375:28)
```

**Stack Trace Points To:**
- `sessionStore.ts:103` - `updateTitle` function
- `ChatDesktop.svelte:136` - Rename session handler
- Svelte reactivity system trying to push to an array

---

## 🔍 Root Cause

**Problem**: Map mutation without creating new reference

The session store uses a `Map<string, ChatSession>` to store sessions. When updating sessions, the code was **mutating the Map in place** without creating a new Map reference:

```typescript
// ❌ WRONG - Mutates in place
sessions.update($sessions => {
  const session = $sessions.get(id);
  if (session) {
    session.title = title;  // Mutate object
    session.updated_at = new Date();
  }
  return $sessions;  // Return same Map reference
});
```

**Why This Breaks:**
1. Svelte's reactivity system detects changes by comparing **references**
2. Returning the same Map reference doesn't trigger reactivity
3. Svelte's internal state gets confused and tries to push to an array
4. This causes an array overflow error (`Invalid array length`)

---

## ✅ Solution

**Create new Map and new session objects on every update:**

```typescript
// ✅ CORRECT - Creates new references
sessions.update($sessions => {
  const session = $sessions.get(id);
  if (session) {
    // Create new Map to trigger Svelte reactivity
    const newSessions = new Map($sessions);
    newSessions.set(id, {
      ...session,           // Spread to create new object
      title,                // Update field
      updated_at: new Date(),
    });
    return newSessions;     // Return NEW Map reference
  }
  return $sessions;
});
```

**Why This Works:**
1. `new Map($sessions)` creates a shallow copy with a new reference
2. `{...session, title}` creates a new session object
3. Svelte detects the reference change and updates properly
4. No array overflow, proper reactivity

---

## 📝 Changes Made

### Functions Fixed

All Map-mutating functions updated to use immutable patterns:

1. **`createSession()`** - Line 87-91
   - Creates new Map before adding session

2. **`updateTitle()`** - Line 103-116
   - Creates new Map and new session object
   - Primary fix for the reported error

3. **`addMessage()`** - Line 124-137
   - Creates new Map when adding messages
   - Spreads session and messages array

4. **`generateTitle()` (2 places)**
   - Line 172-185: Success case
   - Line 197-210: Fallback case
   - Both create new Map references

5. **`deleteSession()` (2 places)**
   - Line 216-221: Main deletion
   - Line 235-246: Fallback creation
   - Both use new Map references

---

## 🎯 Pattern Applied

**Immutable Update Pattern:**
```typescript
sessions.update($sessions => {
  // 1. Get current value
  const item = $sessions.get(id);
  
  if (item) {
    // 2. Create new Map
    const newMap = new Map($sessions);
    
    // 3. Set with new object (spread pattern)
    newMap.set(id, {
      ...item,           // Copy existing fields
      field: newValue,   // Update specific fields
    });
    
    // 4. Return new Map
    return newMap;
  }
  
  return $sessions;
});
```

---

## 🧪 Testing

### Before Fix
```bash
✗ Rename session → RangeError: Invalid array length
✗ Add message → Reactivity doesn't trigger
✗ Update title → State corruption
```

### After Fix
```bash
✓ Rename session works correctly
✓ Messages update in real-time
✓ Titles update immediately
✓ No console errors
✓ Proper reactivity throughout
```

---

## 📚 Svelte Reactivity Rules

### Key Principles

1. **Reference Comparison**: Svelte detects changes by comparing object references
2. **Immutable Updates**: Always create new references for collections (Map, Set, Array)
3. **Spread Pattern**: Use `{...obj, field}` to create new objects
4. **No Mutation**: Never mutate objects/collections directly

### Common Patterns

**Arrays:**
```typescript
// ❌ WRONG
$array.push(item);
$array = $array;

// ✅ CORRECT
$array = [...$array, item];
```

**Objects:**
```typescript
// ❌ WRONG
$object.field = value;
$object = $object;

// ✅ CORRECT
$object = {...$object, field: value};
```

**Maps:**
```typescript
// ❌ WRONG
$map.set(key, value);
$map = $map;

// ✅ CORRECT
$map = new Map($map).set(key, value);
// or
$map = new Map($map);
$map.set(key, value);
$map = $map;
```

---

## 🔗 Related Documentation

- **Svelte Tutorial**: https://svelte.dev/tutorial/updating-arrays-and-objects
- **Svelte Store Guide**: https://svelte.dev/docs#run-time-svelte-store
- **Session Store**: `web/src/lib/stores/sessionStore.ts`

---

## 🎓 Lessons Learned

1. **Always use immutable patterns with Svelte stores**
2. **Map/Set need new references to trigger reactivity**
3. **Spread operator is your friend**: `{...obj}`, `[...arr]`, `new Map(map)`
4. **Test reactivity thoroughly** - subtle bugs are hard to catch
5. **Console errors about arrays might indicate reference issues**

---

## 🚀 Prevention

To prevent similar issues in the future:

1. **ESLint Rule**: Add rule to detect direct Map/Set mutations
2. **Code Review Checklist**: Verify immutable patterns in store updates
3. **Unit Tests**: Test that updates trigger reactivity
4. **Documentation**: Reference this doc when working with stores

---

## ✅ Summary

**Problem**: Mutating Map in place caused Svelte reactivity failure → array overflow  
**Solution**: Create new Map references on every update (immutable pattern)  
**Impact**: All session operations now work correctly with proper reactivity  
**Files Changed**: 1 (`sessionStore.ts`)  
**Lines Changed**: ~40 lines across 7 functions  
**Breaking Changes**: None (internal fix)

---

**Last Updated**: November 3, 2025  
**Version**: SpatialVortex Web UI v0.8.4  
**Bug ID**: sessionstore-001
