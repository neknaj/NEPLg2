---
id: ISS-20260428T200446882Z-RESOURCE-CELLSTATE-MERGE-STARTS-FROM-C81C0269
title: "Resource CellState merge starts from synthetic Uninit"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-29
target: nepl-core/src/resource/cell_state.rs
---

# ISS-20260428T200446882Z-RESOURCE-CELLSTATE-MERGE-STARTS-FROM-C81C0269: Resource CellState merge starts from synthetic Uninit

## 概要

CellTable::merge_paths folds path states from a synthetic Uninit state, so places that stay Initialized on every real branch or loop path become MaybeMoved.

## 対象

- `nepl-core/src/resource/cell_state.rs`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- 親 issue: [ISS-20260428T170745661Z-RESOURCE-IR-RAWMEMORYLOADCELL-GATE-N-1CBE1D0E](./ISS-20260428T170745661Z-RESOURCE-IR-RAWMEMORYLOADCELL-GATE-N-1CBE1D0E.md)
- 一時 `RawMemoryLoadCell` gate で `tests/compiler/move_effect.n.md::doctest#80` が、loop body が raw place を触っていないにもかかわらず false D3100 になった。

## 問題

CellTable::merge_paths folds path states from a synthetic Uninit state, so places that stay Initialized on every real branch or loop path become MaybeMoved.

## 影響

Resource IR Stage 4 reports false D3100/MaybeMoved after branch, loop, or match merges, including raw cells that were not touched by the loop body.

## 修正方針

Fold merge state from the first real path availability_state and only merge remaining real paths; paths where a place is absent still contribute Uninit through availability_state.

## 検証

cargo test -p nepl-core --test resource_ir; temporary RawMemoryLoadCell gate improves tests/compiler/move_effect.n.md loop case from false D3100.

## 対応結果

`CellTable::merge_paths` が実経路ではない `Uninit` を初期値として畳み込んでいたため、全経路で `Initialized(T)` の place でも `MaybeMoved` に落ちていた。最初の実 path の `availability_state` から畳み込みを開始するように変更し、片方の path にしか存在しない place はもう片方の `availability_state` が `Uninit` になる既存の保守性を維持した。

回帰テストとして `resource_ir_cell_check_preserves_raw_cell_across_untouched_loop` を追加し、typechecked source から lowered Resource IR を作ったうえで main function の CellState diagnostics が出ないことを固定した。一時 `RawMemoryLoadCell` gate では `tests/compiler/move_effect.n.md` が 105/110 から 106/110 に改善し、doctest#80 の false D3100 は解消した。残り 4 件は親 issue の realloc ordering / `MemPtr` / helper address summary 系として継続する。
