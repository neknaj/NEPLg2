---
id: ISS-20260428T174427199Z-RESOURCE-CELLSTATE-RAW-ADDRESS-ALIAS-45DC270E
title: "Resource CellState raw address aliases do not cross helper returns and aggregate fields"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: nepl-core/src/resource/initialized.rs
---

# ISS-20260428T174427199Z-RESOURCE-CELLSTATE-RAW-ADDRESS-ALIAS-45DC270E: Resource CellState raw address aliases do not cross helper returns and aggregate fields

## 概要

Resource IR CellState tracks raw memory cells by canonical raw address, but raw address aliases are only local copies. A helper that returns a raw address parameter, a function value call, or an aggregate field such as MemPtr.ptr loses the alias, so a store through one spelling and load through another becomes a false RawMemoryLoadCell uninitialized diagnostic.

## 対象

- `nepl-core/src/resource/initialized.rs`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- 親 issue: [ISS-20260428T170745661Z-RESOURCE-IR-RAWMEMORYLOADCELL-GATE-N-1CBE1D0E](./ISS-20260428T170745661Z-RESOURCE-IR-RAWMEMORYLOADCELL-GATE-N-1CBE1D0E.md)
- `RawMemoryLoadCell` gate の一時調査で、helper-returned slot と `MemPtr` / `RegionToken` style wrapper の raw address alias が CellState 側へ渡らず、store 済み cell が別名 load で uninitialized になる false positive が残っていた。

## 問題

Resource IR CellState tracks raw memory cells by canonical raw address, but raw address aliases are only local copies. A helper that returns a raw address parameter, a function value call, or an aggregate field such as MemPtr.ptr loses the alias, so a store through one spelling and load through another becomes a false RawMemoryLoadCell uninitialized diagnostic.

## 影響

RawMemoryLoadCell cannot be made authoritative for Stage 4 because correct programs using small pointer helpers or MemPtr-style wrappers fail before the intended ownership/effect diagnostics. This keeps old HIR raw summaries alive and blocks the static check simplification plan.

## 修正方針

Add Resource IR raw address return summaries for direct and function-value calls, propagate raw address aliases through aggregate fields, and keep canonical raw cell keys stable across helper-returned aliases.

## 修正内容

- `check_resource_initialized_moves` の前段で raw address return summary を fixed point 計算し、direct call と known function-value indirect call の戻り値が raw address 引数の alias である場合に CellState の canonical address alias へ反映した。
- `Construct` / branch / match の結果について、aggregate field に入った raw address alias と function alias を維持するようにした。
- `RawCellAddressAliases::copy_alias_or_seed` を prefix replacement ベースにし、aggregate field descendant の alias が wrapper copy / branch result / helper result を越えて失われないようにした。

## 検証

Add Resource IR CellState regressions for helper-returned raw slot load, function-value helper raw slot load, and aggregate-field raw address load. Run cargo test -p nepl-core --test resource_ir, cargo check -p nepl-core --tests, trunk build, and focused move_effect/move_check doctests.

- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_preserves_raw_address -- --nocapture`: 3 passed
- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 73 passed
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc\tests.js -i tests\compiler\move_effect.n.md --no-tree -o tmp\resource-alias-summary-move-effect.json -j 1`: total=110, passed=110
- `node nodesrc\tests.js -i tests\compiler\move_check.n.md --no-tree -o tmp\resource-alias-summary-move-check.json -j 1`: total=52, passed=52
