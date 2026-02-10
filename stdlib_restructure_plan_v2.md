
# Stdlib Restructuring Plan (Revised)

Based on user feedback and dependency analysis, we will move OS-independent modules to `alloc` (or `core`).

## Target Structure (Revised)

```text
stdlib/
  core/                  # Pure logic, no allocation, no OS
    math.nepl
    mem.nepl
    option.nepl
    result.nepl
    cast.nepl
    rand/
      xorshift32.nepl    # Moved to core (pure)
    
  alloc/                 # Heap-dependent, no OS
    vec.nepl
    string.nepl
    collections/
      list.nepl
      stack.nepl
      hashmap.nepl
      hashset.nepl
      btreemap.nepl
      btreeset.nepl
    encoding/
      json.nepl          # Moved to alloc
    diag/
      error.nepl         # Moved to alloc
      diag.nepl          # Moved to alloc
    hash/
      fnv1a32.nepl       # Moved to alloc (could be core, but kept with sha256)
      sha256.nepl        # Moved to alloc (uses Vec)
      
  std/                   # OS / IO / Environment
    stdio.nepl
    test.nepl
    env/
      cliarg.nepl
```

## Action Steps

1. **Directories**: Ensure `core/rand`, `alloc/encoding`, `alloc/diag`, `alloc/hash` exist.
2. **Moves**:
   - `std/rand` -> `core/rand`
   - `std/hash` -> `alloc/hash`
   - `std/encoding/json.nepl` -> `alloc/encoding/json.nepl` (Already creating/moving in previous step, checking logic)
   - `std/diag` (dir) -> `alloc/diag` (Already planned, verifying)

   *Wait*, in the previous step I planned:
   - `json` -> `std/encoding/json` (Incorrect, should be `alloc`)
   - `error/diag` -> `std/diag` (Incorrect, should be `alloc`)
   - `hash` -> `std/hash` (Incorrect, should be `alloc`)
   - `rand` -> `std/rand` (Incorrect, should be `core`)

   I will correct these destinations in the execution.

3. **Update Imports**: Update tests to reflect these new locations.
