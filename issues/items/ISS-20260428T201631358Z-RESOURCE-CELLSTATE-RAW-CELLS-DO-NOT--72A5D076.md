---
id: ISS-20260428T201631358Z-RESOURCE-CELLSTATE-RAW-CELLS-DO-NOT--72A5D076
title: "Resource CellState raw cells do not rekey when address alias canonical changes"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-29
target: "nepl-core/src/resource/initialized.rs, nepl-core/src/resource/cell_state.rs"
---

# ISS-20260428T201631358Z-RESOURCE-CELLSTATE-RAW-CELLS-DO-NOT--72A5D076: Resource CellState raw cells do not rekey when address alias canonical changes

## 概要

Resource IR CellState stores raw memory cell state under the raw address canonical place, but let/read/assign/move alias transfers can change the canonical address without moving existing raw cell entries.

## 対象

- `nepl-core/src/resource/initialized.rs, nepl-core/src/resource/cell_state.rs`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- 親 issue: [ISS-20260428T170745661Z-RESOURCE-IR-RAWMEMORYLOADCELL-GATE-N-1CBE1D0E](./ISS-20260428T170745661Z-RESOURCE-IR-RAWMEMORYLOADCELL-GATE-N-1CBE1D0E.md)
- 一時 `RawMemoryLoadCell` gate で `tests/compiler/move_effect.n.md::doctest#8` が、`realloc_raw` 後の raw slot load で D3025 ではなく false D3100 になった。

## 問題

Resource IR CellState stores raw memory cell state under the raw address canonical place, but let/read/assign/move alias transfers can change the canonical address without moving existing raw cell entries.

## 影響

RawMemoryLoadCell reports false D3100 after realloc_raw output is bound to a local, so the intended D3025 raw allocation escape diagnostic is hidden.

## 修正方針

When ResourceCheckEngine copies or seeds a raw address alias, compute the old and new canonical address and rekey CellTable raw cell entries if the canonical place changes.

## 検証

cargo test -p nepl-core --test resource_ir resource_ir_cell_check_realloc_transfers_copy_raw_cells; temporary RawMemoryLoadCell gate improves tests/compiler/move_effect.n.md from 106/110 to 107/110.

## 対応結果

`realloc_raw` の Resource IR `RawMemory::Realloc` は Copy raw cell を output temporary へ transfer していたが、直後の `let grown = tmp` で raw address canonical が local `grown` へ変わった時に `CellTable` の raw cell entry が `tmp.deref` のまま残っていた。`ResourceCheckEngine` の raw address alias transfer を `copy_raw_alias_and_rekey_cells` へ統一し、alias canonical が変化した場合に `CellTable::rekey_raw_cells` で raw cell state を旧 canonical から新 canonical へ移すようにした。

回帰テストとして `resource_ir_cell_check_realloc_transfers_copy_raw_cells` を追加し、`realloc_raw` 後に local 束縛された raw slot から `load_i32` しても initialized/moved CellState diagnostic が出ないことを固定した。一時 `RawMemoryLoadCell` gate では `tests/compiler/move_effect.n.md` が 106/110 から 107/110 に改善し、doctest#8 は本来の D3025 raw allocation escape diagnostic へ戻った。残り 3 件は `MemPtr` / `RegionToken` wrapper address summary と literal helper address summary として継続する。
