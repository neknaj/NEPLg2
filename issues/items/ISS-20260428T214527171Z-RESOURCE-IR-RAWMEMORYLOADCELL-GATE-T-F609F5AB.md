---
id: ISS-20260428T214527171Z-RESOURCE-IR-RAWMEMORYLOADCELL-GATE-T-F609F5AB
title: "Resource IR RawMemoryLoadCell gate treats external raw parameters as uninitialized"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-29
target: "nepl-core/src/resource/initialized.rs, nepl-core/src/resource/cell_state.rs, nepl-core/src/resource/initialized_raw_memory.rs, nepl-core/src/resource/initialized_alias.rs, nepl-core/src/resource/lower.rs, nepl-core/tests/resource_ir.rs, tests/tutorials"
---

# ISS-20260428T214527171Z-RESOURCE-IR-RAWMEMORYLOADCELL-GATE-T-F609F5AB: Resource IR RawMemoryLoadCell gate treats external raw parameters as uninitialized

## 概要

After RawMemoryLoadCell became a compiler gate, tutorial doctests fail because loads from initialized external raw addresses such as str parameters and std/test result buffers are reported as Uninit raw cells.

## 対象

- `nepl-core/src/resource/initialized.rs, nepl-core/src/resource/cell_state.rs, nepl-core/src/resource/initialized_raw_memory.rs, nepl-core/tests/resource_ir.rs, tests/tutorials`

## 根拠

- `node nodesrc/tests.js -i tutorials --no-tree -o tmp/raw-load-cell-external-roots-tutorials.json -j 1` で `len__str__i32__pure` の `str_addr` 由来 load と `checks_has_err_loop` の external raw data pointer load が `RawMemoryLoadCell` Uninit になった。
- [doc/neplg2/static_check_complexity_reduction_plan.md](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 4 の `initialized / moved state` 移行中に、compiler-owned allocation と外部から渡された初期化済み raw storage を同じ `Uninit` として扱っていた。

## 問題

After RawMemoryLoadCell became a compiler gate, tutorial doctests fail because loads from initialized external raw addresses such as str parameters and std/test result buffers are reported as Uninit raw cells.

## 影響

Stage 4 raw load enforcement blocks valid stdlib/tutorial code and forces a false choice between disabling the gate and weakening raw load-before-store diagnostics.

## 修正方針

Separate function-external initialized raw roots from compiler-owned uninitialized storage in Resource IR CellState. Seed parameters as external roots, keep alloc/realloc storage as known-uninitialized until store initializes a typed cell, and only report RawMemoryLoadCell Uninit for known compiler-owned storage or previously tracked cells.

## 対応

- `CellTable` に compiler-owned raw storage root と function-external initialized raw storage root を分離して持たせた。
- `alloc_raw` / owned `realloc_raw` は owned root として扱い、store されるまでは load-before-store を D3100 として維持した。
- 関数 parameter は external initialized root として seed し、未追跡 external root からの raw load は許可する。ただし non-Copy load 後は raw cell を `Moved` として記録し、二重 load は拒否する。
- `str_addr` と direct `add` / `sub` / `mem_ptr_*` / `region_new` の raw address lowering を Resource IR alias として露出した。
- `RawCellAddressAliases` は `StorageOffset` 派生 alias を base alias group に混ぜず、`p` と `p + n` を別 raw address として保持するようにした。

## 検証

- `rustfmt --check nepl-core/src/resource/cell_state.rs nepl-core/src/resource/initialized.rs nepl-core/src/resource/initialized_alias.rs nepl-core/src/resource/initialized_raw_memory.rs nepl-core/src/resource/lower.rs nepl-core/tests/resource_ir.rs`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check -- --nocapture`: 24 passed
- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 86 passed
- `cargo check -p nepl-core --tests`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_preserves_str_addr_helper_parameter_raw_load -- --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_preserves_direct_arithmetic_external_raw_load -- --nocapture`: pass
- `trunk build`: pass
- `node nodesrc/tests.js -i tutorials/getting_started/01_hello_world.n.md --no-tree -o tmp/raw-load-cell-hello-world-after-main-sync.json -j 1`: 1 passed
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/raw-load-cell-move-effect-after-main-sync.json -j 1`: 110 passed
- `node nodesrc/tests.js -i tests/compiler/move_check.n.md --no-tree -o tmp/raw-load-cell-move-check-after-main-sync.json -j 1`: 52 passed
- `node nodesrc/tests.js -i tutorials --no-tree -o tmp/raw-load-cell-external-roots-after-main-sync-tutorials.json -j 1`: 2 passed / 22 failed。`len__str__i32__pure`、`checks_has_err_loop`、`checks_summary_loop` の external raw root false positive は消えた。残りは `RegionToken` / allocation scratch cell / `Vec get_ref` の raw cell moved/uninit issue として既存の MemPtr/RegionToken provenance/owner 分離系 issue で扱う。
- `node nodesrc/issues.js check`: pass
