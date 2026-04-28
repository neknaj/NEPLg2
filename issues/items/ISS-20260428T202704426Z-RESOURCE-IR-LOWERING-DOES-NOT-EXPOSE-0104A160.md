---
id: ISS-20260428T202704426Z-RESOURCE-IR-LOWERING-DOES-NOT-EXPOSE-0104A160
title: "Resource IR lowering does not expose MemPtr and RegionToken wrapper aliases"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-29
target: "nepl-core/src/resource/lower.rs, nepl-core/src/resource/initialized_alias.rs"
---

# ISS-20260428T202704426Z-RESOURCE-IR-LOWERING-DOES-NOT-EXPOSE-0104A160: Resource IR lowering does not expose MemPtr and RegionToken wrapper aliases

## 概要

RawMemoryLoadCell gate still reports false D3100 for MemPtr disjoint offsets and RegionToken load-then-dealloc cases because Resource IR treats mem_ptr_wrap, mem_ptr_addr, mem_ptr_add, and region_new as opaque calls instead of structural pointer wrapper operations.

## 対象

- `nepl-core/src/resource/lower.rs, nepl-core/src/resource/initialized_alias.rs`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- 親 issue: [ISS-20260428T170745661Z-RESOURCE-IR-RAWMEMORYLOADCELL-GATE-N-1CBE1D0E](./ISS-20260428T170745661Z-RESOURCE-IR-RAWMEMORYLOADCELL-GATE-N-1CBE1D0E.md)
- 一時 `RawMemoryLoadCell` gate で `tests/compiler/move_effect.n.md::doctest#23` と `#38` が、`mem_ptr_addr` の戻り値 temporary を store/load 間で別 raw address と見て false D3100 になった。
- `core/mem` の `MemPtr<.T>` は `raw <i32>` を持つ non-owning pointer wrapper、`RegionToken<.T>` は `ptr <MemPtr<.T>>` と `size <i32>` を持つ storage token であり、Resource IR はこの構造的 projection を CellState に渡す必要がある。

## 問題

RawMemoryLoadCell gate still reports false D3100 for MemPtr disjoint offsets and RegionToken load-then-dealloc cases because Resource IR treats mem_ptr_wrap, mem_ptr_addr, mem_ptr_add, and region_new as opaque calls instead of structural pointer wrapper operations.

## 影響

RawMemoryLoadCell cannot become authoritative for Stage 4: valid MemPtr and RegionToken wrapper programs fail before the intended ownership diagnostics, keeping old HIR raw summaries necessary.

## 修正方針

Lower core/mem pointer wrapper helpers into explicit Resource IR structural alias operations while preserving call/effect coverage, so MemPtr.raw and RegionToken.ptr.raw aliases are available to CellState.

## 検証

Add resource_ir regression for MemPtr disjoint offset and RegionToken load-then-dealloc alias preservation; temporary RawMemoryLoadCell gate should improve tests/compiler/move_effect.n.md.

## 対応結果

`ResourceOp::RawAddressAlias` を追加し、Resource IR の call/effect coverage count を変えずに raw address alias だけを checker へ渡せるようにした。`lower.rs` は `mem_ptr_wrap` / `mem_ptr_addr` / `mem_ptr_add` / `region_new` を補助 lowering し、`MemPtr.raw` と `RegionToken.ptr` の raw address projection を `CellState` に渡す。

同時に、whole wrapper value の copy/read 時に `CellTable::rekey_raw_cells` が wrapper root 配下の raw cell を temporary 側へ移してしまう問題も修正した。rekey は exact な raw address alias を追跡している場合だけ行い、`MemPtr` root copy では既存 raw cell state を移動しない。

回帰テストとして `resource_ir_cell_check_preserves_mem_ptr_disjoint_offsets` と `resource_ir_cell_check_preserves_mem_ptr_alias_after_region_token` を追加した。一時 `RawMemoryLoadCell` gate では `tests/compiler/move_effect.n.md` が 107/110 から 109/110 に改善し、`doctest#23` と `#38` の false D3100 は解消した。残件 `doctest#30` は `ISS-20260428T203931325Z-RESOURCE-IR-RAW-ADDRESS-SUMMARIES-DO-C7473DEA` として分離した。
