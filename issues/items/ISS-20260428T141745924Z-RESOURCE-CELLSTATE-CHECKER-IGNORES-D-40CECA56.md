---
id: ISS-20260428T141745924Z-RESOURCE-CELLSTATE-CHECKER-IGNORES-D-40CECA56
title: "Resource CellState checker ignores destructive raw storage operations"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/resource/check.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260428T141745924Z-RESOURCE-CELLSTATE-CHECKER-IGNORES-D-40CECA56: Resource CellState checker ignores destructive raw storage operations

## 概要

Resource IR CellState now models raw load/store slots, but dealloc/realloc/fill/bulk copy still only check ordinary arguments. An initialized non-Copy payload stored in a raw cell can therefore be freed, reallocated, byte-filled, or copied as storage without a ResourceCheck diagnostic.

## 対象

- `nepl-core/src/resource/check.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- `nepl-core/src/resource/check.rs` の `check_raw_memory` は `Load` / `Store` 以外の raw memory op を `RawMemoryArgument` として扱い、`address.*` の initialized cell を確認していなかった。
- 既存の旧 `move_check` は D3100 で dealloc / realloc / bulk copy / byte write の live non-Copy payload を拒否しているため、Resource IR 側が同じ安全境界を持たないと Stage 4 の authoritative 化に進めない。

## 問題

Resource IR CellState now models raw load/store slots, but dealloc/realloc/fill/bulk copy still only check ordinary arguments. An initialized non-Copy payload stored in a raw cell can therefore be freed, reallocated, byte-filled, or copied as storage without a ResourceCheck diagnostic.

## 影響

Stage 4 cannot become authoritative while storage-only raw operations can erase initialized/maybe-moved cells. The old move_check D3100 guards would have to remain as ad-hoc HIR summaries, preserving the complexity this migration is meant to remove.

## 修正方針

Treat destructive raw memory operations as cell-state operations: dealloc/realloc reject live non-Copy cells under the freed address, fill and bulk destination reject overwriting live cells, and bulk source rejects copying live cells. Keep payload-consumed storage-only dealloc allowed.

## 検証

Add Resource IR unit tests for dealloc/realloc/fill/bulk copy diagnostics and storage-only dealloc after non-Copy load. Run focused resource_ir tests and issue index validation.

## 2026-04-28 修正

`check_resource_initialized_moves` の `RawMemoryOp` 処理を raw storage destructive operation として扱うようにした。

- `Store` は destination raw cell に live non-Copy / maybe-moved obligation が残っている場合、`RawMemoryStoreCell` として拒否する。
- `Dealloc` / `Realloc` / `Fill` は address value と、その address 配下の raw cell obligation を分けて検査する。
- `BulkCopy` / `BulkMove` は destination overwrite と source shallow copy の両方で live non-Copy cell を拒否する。
- non-Copy `Load` で payload を consume 済みの raw cell は storage-only dealloc を許可する。
- Copy initialized raw cell は `Realloc` / `BulkCopy` で destination 側へ引き継ぎ、`Fill` / `Dealloc` では raw cell state を消す。

この修正は `MemPtr` を owner として拡張するものではなく、`MemPtr = non-owning pointer`、initialized payload = `address.*` cell state、storage free obligation = owner checker という分離を Resource IR 側へ進める Stage 4 の修正である。

## 回帰テスト

- `resource_ir_cell_check_reports_store_over_live_raw_cell`
- `resource_ir_cell_check_allows_dealloc_after_non_copy_raw_load`
- `resource_ir_cell_check_reports_destructive_raw_storage_ops_over_live_cell`
- `resource_ir_cell_check_reports_bulk_copy_of_live_non_copy_raw_cells`

確認:

- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_ -- --nocapture`
- `cargo test -p nepl-core --test resource_ir -- --nocapture`
- `rustfmt --check nepl-core\src\resource\check.rs nepl-core\tests\resource_ir.rs`
- `node nodesrc/issues.js check`
- `git diff --check`
- `trunk build`
