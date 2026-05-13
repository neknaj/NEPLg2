---
id: ISS-20260513T023254911Z-CORE-MEM-FACADE-STILL-CARRIED-RAW-ME-FEEF633F
title: "core mem facade still carried raw memory boundary privilege"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-13
updated: 2026-05-13
target: "stdlib/core/mem.nepl, stdlib/core/mem/**, nepl-core/src/loader.rs"
---

# ISS-20260513T023254911Z-CORE-MEM-FACADE-STILL-CARRIED-RAW-ME-FEEF633F: core mem facade still carried raw memory boundary privilege

## 概要

core/mem.nepl acted both as the public facade and as the raw-memory-boundary implementation file, so the facade itself carried raw memory capability and all allocator/raw/pointer responsibilities stayed in one large file.

## 対象

- `stdlib/core/mem.nepl, stdlib/core/mem/**, nepl-core/src/loader.rs`

## 根拠

- `stdlib/core/mem.nepl` は 1192 行あり、public facade、`MemPtr` / `RegionToken` 型、allocator、raw load/store、checked pointer wrapper、doctest が同居していた。
- `nepl-core/src/loader.rs` の `RAW_MEMORY_BOUNDARY_STDLIB_PATHS` は root facade である `core/mem.nepl` 自体に `SourceCapability::RawMemoryBoundary` を付けていた。
- この形では import visibility enforcement が実装済みでも、Stage 6 で raw-memory-boundary capability を public facade から狭められない。

## 問題

core/mem.nepl acted both as the public facade and as the raw-memory-boundary implementation file, so the facade itself carried raw memory capability and all allocator/raw/pointer responsibilities stayed in one large file.

## 影響

Stage 6 cannot narrow raw memory authority while the public facade owns raw bodies and receives SourceMap RawMemoryBoundary capability.

## 修正方針

Split core/mem into facade plus typed submodules for types, raw operations, allocator, and pointer wrappers. Move raw-memory-boundary capability from the facade to exact implementation submodule paths and add source policy coverage.

## 検証

Run core/mem focused doctests, loader tests, source policy, and issues check.

## 2026-05-13 修正

`stdlib/core/mem.nepl` を public facade に縮小し、実装責務を次の submodule へ分離した。

- `stdlib/core/mem/types.nepl`: `MemPtr<T>` / `RegionToken<T>` と non-owning pointer view helper。
- `stdlib/core/mem/raw.nepl`: raw `load` / `store` / `mem_copy` / `mem_move` / `memset` / `size_of` / `align_of` と target raw body。
- `stdlib/core/mem/allocator.nepl`: free list allocator、checked raw allocator wrapper、compiler runtime ABI。
- `stdlib/core/mem/pointer.nepl`: `RegionToken` / `MemPtr` の checked allocation、projection、load/store/copy wrapper。

loader の exact raw-memory-boundary table から `core/mem.nepl` を外し、上記 4 submodule だけに capability を付与した。これにより public facade 自体は raw body / raw intrinsic / function body を持たず、raw authority は実装 module の exact path に限定される。

退行防止として `nodesrc/test_stdlib_core_mem_boundary.js` を追加し、root facade に関数本体や raw body が戻らないこと、loader table が facade ではなく exact submodule を指すこと、各 submodule の責務と行数上限を監視する。

この修正は root facade の raw-memory-boundary privilege を外す対応であり、`alloc_raw` / `mem_ptr_addr` など raw helper の public re-export を閉じる最終対応ではない。safe public API migration は親 issue `ISS-20260427T152954558Z-CORE-MEM-EXPOSES-RAW-ADDRESS-ESCAPE--4185EA5D` と `ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84` で継続する。

検証:

- `cargo fmt -p nepl-core --check`: pass。
- `cargo check -p nepl-core --tests`: pass。
- `cargo test -p nepl-core --test import_clause -- --nocapture`: 17/17 pass。
- `trunk build`: pass。
- `node nodesrc/test_stdlib_core_mem_boundary.js`: pass。
- `node nodesrc/test_static_check_boundary_responsibility.js`: pass。
- `node nodesrc/tests.js -i stdlib/core/mem.nepl -i stdlib/core/mem/types.nepl -i stdlib/core/mem/raw.nepl -i stdlib/core/mem/allocator.nepl -i stdlib/core/mem/pointer.nepl --no-tree -o tmp/agent1-core-mem-split-doctests.json -j 1 --dist web/dist`: 6/6 pass。
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/agent1-core-mem-split-memory-safety.json -j 1 --dist web/dist`: 23/23 pass。
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/agent1-core-mem-split-move-effect.json -j 1 --dist web/dist`: 113/113 pass。
