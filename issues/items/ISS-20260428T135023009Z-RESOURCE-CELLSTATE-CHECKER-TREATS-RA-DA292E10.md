---
id: ISS-20260428T135023009Z-RESOURCE-CELLSTATE-CHECKER-TREATS-RA-DA292E10
title: "Resource CellState checker treats raw memory load and store as ordinary arguments"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/resource/check.rs, nepl-core/tests/resource_ir.rs, doc/neplg2/static_check_complexity_reduction_plan.md"
---

# ISS-20260428T135023009Z-RESOURCE-CELLSTATE-CHECKER-TREATS-RA-DA292E10: Resource CellState checker treats raw memory load and store as ordinary arguments

## 概要

Resource IR lowering records raw memory load and store as ResourceOp::RawMemory, but check_resource_initialized_moves only ensures the pointer/value arguments and marks the output initialized. It does not model the pointed-to cell as initialized, moved, or unavailable, so a raw load before store or repeated non-Copy load is invisible to the Resource IR CellState checker.

## 対象

- `nepl-core/src/resource/check.rs, nepl-core/tests/resource_ir.rs, doc/neplg2/static_check_complexity_reduction_plan.md`

## 根拠

- `nepl-core/src/resource/check.rs` の `ResourceOp::RawMemory` handling は、operation 種別を見ずに args の availability を確認し、output を initialized にするだけだった。
- そのため `RawMemoryOp::Load` は pointer が指す cell を確認せず、`RawMemoryOp::Store` も pointer が指す cell を initialized state にしなかった。
- `CellTable::availability_state` は aggregate projection と memory projection を同じ prefix relation として扱っており、pointer value が initialized なら `ptr.*` も initialized と誤判定し得た。

## 問題

Resource IR lowering records raw memory load and store as ResourceOp::RawMemory, but check_resource_initialized_moves only ensures the pointer/value arguments and marks the output initialized. It does not model the pointed-to cell as initialized, moved, or unavailable, so a raw load before store or repeated non-Copy load is invisible to the Resource IR CellState checker.

## 影響

Stage 4 cannot become authoritative for initialized/moved state while raw memory slots are outside CellState. Memory-safety cases still need the old HIR move_check raw-place summary, and Resource IR shadow checks can remain clean even when raw load/store would read uninitialized storage or duplicate non-Copy payloads.

## 修正方針

Model raw memory slots as pointer-place deref projections in Resource CellState. Store consumes the value and initializes the pointed cell; Load checks the pointed cell, moves it for non-Copy payloads, and initializes the output. Add focused Resource IR regressions for uninitialized raw load and repeated non-Copy raw load.

## 修正内容

- `RawMemoryOp::Load` は address value と `address.*` cell を別々に確認し、non-Copy payload の load では `address.*` を `Moved` にするようにした。
- `RawMemoryOp::Store` は address value を確認し、store value を by-value consume してから `address.*` cell を initialized にするようにした。
- `CellTable` の projection availability を修正し、`Field` / `TupleField` / `EnumPayload` は aggregate cell state として伝播し、`StorageOffset` は address value の availability だけを継承し、`Deref` は pointer value と pointee cell の境界として扱うようにした。
- `resource_ir_cell_check_reports_raw_load_before_store` と `resource_ir_cell_check_moves_non_copy_raw_load_cell` を追加し、未初期化 raw load と non-Copy raw load の二重 move を Resource IR CellState で検出することを固定した。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_reports_raw_load_before_store -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_moves_non_copy_raw_load_cell -- --nocapture`
- `cargo test -p nepl-core --test resource_ir -- --nocapture`
- commit 前に `trunk build`、`node nodesrc/issues.js check`、`rustfmt --check`、`git diff --check` を実行する。

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
