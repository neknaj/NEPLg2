
# Stdlib Restructuring Plan

We will reorganize the flat `std/` directory into a layered structure: `core`, `alloc`, `std`.

## Target Structure

```text
stdlib/
  core/                  # Pure logic, no allocation, no OS
    math.nepl
    mem.nepl
    option.nepl
    result.nepl
    cast.nepl           # Utility
    
  alloc/                 # Heap-dependent
    vec.nepl
    string.nepl
    box.nepl            # if needed
    collections/
      list.nepl
      stack.nepl
      hashmap.nepl
      hashset.nepl
      btreemap.nepl
      btreeset.nepl
      
  std/                   # OS / IO / Environment
    stdio.nepl
    test.nepl
    diag/
      error.nepl
      diag.nepl
    env/
      cliarg.nepl
    encoding/
      json.nepl
    hash/
      fnv1a32.nepl
      sha256.nepl
    rand/
      xorshift32.nepl
```

## Moving Plan

1. **Create Directories**:
   - `stdlib/core`
   - `stdlib/alloc`
   - `stdlib/alloc/collections`
   - `stdlib/std/env`
   - `stdlib/std/encoding`
   - `stdlib/std/diag`

2. **Move Files**:

   | Source | Destination |
   | :--- | :--- |
   | `std/math.nepl` | `core/math.nepl` |
   | `std/mem.nepl` | `core/mem.nepl` |
   | `std/option.nepl` | `core/option.nepl` |
   | `std/result.nepl` | `core/result.nepl` |
   | `std/cast.nepl` | `core/cast.nepl` |
   | `std/vec.nepl` | `alloc/vec.nepl` |
   | `std/string.nepl` | `alloc/string.nepl` |
   | `std/list.nepl` | `alloc/collections/list.nepl` |
   | `std/stack.nepl` | `alloc/collections/stack.nepl` |
   | `std/hashmap.nepl` | `alloc/collections/hashmap.nepl` |
   | `std/hashset.nepl` | `alloc/collections/hashset.nepl` |
   | `std/btreemap.nepl` | `alloc/collections/btreemap.nepl` |
   | `std/btreeset.nepl` | `alloc/collections/btreeset.nepl` |
   | `std/cliarg.nepl` | `std/env/cliarg.nepl` |
   | `std/json.nepl` | `std/encoding/json.nepl` |
   | `std/error.nepl` | `std/diag/error.nepl` |
   | `std/diag.nepl` | `std/diag/diag.nepl` |
   | `std/hash/*` | `std/hash/*` (Keep but ensure it is in correct place) |
   | `std/rand/*` | `std/rand/*` (Keep) |
   | `std/stdio.nepl` | `std/stdio.nepl` (Keep) |
   | `std/test.nepl` | `std/test.nepl` (Keep) |

3. **Update Imports**:
   - We need to be careful. Since the user asked to "structure nicely", breaking existing imports in `tests/*.nepl` or other files is a risk.
   - **However**, since this is a "rebuild from scratch" style request for the standard library structure, updating usage is implied or we should provide re-exports.
   - **User Request**: "stdlibのファイルを良い感じに構造化して整理してみて" (Structure and organize stdlib files nicely).
   - I will move the files. The user will likely need to update imports in their user code or I should update `stdlib/tests/*.nepl`.

## Action Steps

1. Create target folders.
2. Move files using `run_command` (powershell `mv`).
3. Update `stdlib/std.nepl` (if it exists, likely need to create one if sticking to the "facade" pattern) or just let the folder structure speak.
   - Currently `stdlib/std` is a directory.
   - The user imports e.g. `std/math`.
   - After move, it would be `core/math` or `alloc/vec`.
   - To preserve backward compat, we might leave "forwarding" files in `std/`, but for a clean break, we should just move them and update the tests.
   - I will check `stdlib/tests/*.nepl` and update them.

