---
id: ISS-20260505T092815045Z-RESOURCE-IR-FILL-INITIALIZATION-COLL-22447391
title: "Resource IR fill initialization collapses range facts to dynamic cells"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-05
updated: 2026-05-05
target: "nepl-core/src/resource/initialized_raw_memory.rs, nepl-core/src/resource/cell_state.rs, nepl-core/src/resource/initialized_summary.rs, nepl-core/tests/resource_ir.rs, tests/stdlib/memory_safety.n.md"
---

# ISS-20260505T092815045Z-RESOURCE-IR-FILL-INITIALIZATION-COLL-22447391: Resource IR fill initialization collapses range facts to dynamic cells

## 概要

After ResourceOffset exact/dynamic separation, RawMemoryOp::Fill still records initialized Copy cells only as a dynamic-offset cell. Exact raw loads such as load_i32 add p 0 and checked MemPtr fill_i32 Ok arms can no longer prove initialization and fail with resource.cell.uninit.

## 対象

- `nepl-core/src/resource/initialized_raw_memory.rs, nepl-core/src/resource/cell_state.rs, nepl-core/src/resource/initialized_summary.rs, nepl-core/tests/resource_ir.rs, tests/stdlib/memory_safety.n.md`

## 根拠

- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_raw_fill -- --nocapture` が `load_u8 add p 0` / `load_i32 add p 12` など exact offset load を `resource.cell.uninit` として失敗していた。
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_allows_region_ptr_rewrap_view_dealloc -- --nocapture` も、`cleanup_raw = mem_ptr_addr p_u8` を先に作ると canonical raw alias が変わり、`fill_i32 p_i32 4 7` の `Result::Ok` 後に `load_i32 p_i32` が initialized と証明できなかった。
- `tests/stdlib/memory_safety.n.md::doctest#8` は同じ問題を source-level に露出し、`resource.cell.uninit` で失敗していた。

## 問題

After ResourceOffset exact/dynamic separation, RawMemoryOp::Fill still records initialized Copy cells only as a dynamic-offset cell. Exact raw loads such as load_i32 add p 0 and checked MemPtr fill_i32 Ok arms can no longer prove initialization and fail with resource.cell.uninit.

## 影響

Valid initialized Copy memory reads after memset_u8/fill_i32 are rejected. This breaks memory_safety doctest#8 and existing Resource IR regression tests, while weakening the dynamic/exact distinction would reintroduce unsound unknown-offset proofs.

## 修正方針

Represent fill operations with typed fill units and exact/range-aware initialized facts. Direct fill with known count and Result::Ok-gated checked MemPtr fill summaries must initialize only offsets proven to be inside the fill range.

## 対応

- `RawMemoryOp::Fill` を `RawMemoryFillUnit` 付きにし、`memset_u8` / `fill_u8` は byte fill、`fill_i32` は i32 stride fill として lowering するようにした。
- `CellTable` に known-count raw fill range fact を追加し、exact raw load が fill range 内かつ型一致する場合だけ initialized とみなすようにした。
- dynamic offset の initialized fact を exact offset の証明として再利用しない方針は維持し、unknown-offset non-Copy load の保守性を壊していない。
- checked MemPtr wrapper の `Result::Ok` gated summary に fill range fact を伝播し、caller 側の literal count と current alias set から range を materialize するようにした。
- alias canonical が `cleanup_raw` のような別 local に寄っても `p_i32.raw` 側の exact load が証明できるよう、fill range は現在の raw address aliases 全体へ記録する。

## 検証

- `cargo fmt --check -p nepl-core`: pass
- `cargo check -p nepl-core --tests`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_raw_fill -- --nocapture`: 2 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_allows_region_ptr_rewrap_view_dealloc -- --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_unknown_offset_load_does_not_clear_all_exact_cells -- --nocapture`: pass
- `trunk build`: pass
- `node nodesrc/run_doctest.js -i tests/stdlib/memory_safety.n.md -n 8 --dist web/dist`: pass
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/memory-safety-fill-range-agent1-after.json -j 1 --dist web/dist`: 12 total / 12 passed
- `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 80 --dist web/dist`: pass
