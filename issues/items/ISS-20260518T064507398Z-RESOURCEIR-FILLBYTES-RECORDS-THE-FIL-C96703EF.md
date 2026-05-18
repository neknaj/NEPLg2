---
id: ISS-20260518T064507398Z-RESOURCEIR-FILLBYTES-RECORDS-THE-FIL-C96703EF
title: "ResourceIR byte raw memory operations lose the u8 cell type proof"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "nepl-core/src/resource/initialized_raw_fill.rs; nepl-core/tests/resource_ir.rs; tests/stdlib/memory_safety.n.md"
---

# ISS-20260518T064507398Z-RESOURCEIR-FILLBYTES-RECORDS-THE-FIL-C96703EF: ResourceIR byte raw memory operations lose the u8 cell type proof

## 概要

RawMemoryOp::FillBytes and load_u8/store_u8 did not preserve the memory cell type separately from the ABI value type. The public memset_u8/fill_u8 helpers pass and return byte values as i32, so a later load_u8 from the filled MemPtr was rejected as resource.cell.uninit even though the source-level checked helper had initialized that byte range.

## 対象

- `nepl-core/src/resource/initialized_raw_fill.rs; nepl-core/tests/resource_ir.rs; tests/stdlib/memory_safety.n.md`

## 根拠

- `tests/stdlib/memory_safety.n.md::doctest#14` rejected `fill_u8 p 16 7` followed by `load_u8 p` with `resource.cell.uninit`.
- The original FillBytes implementation recorded the initialized byte range with the fill value expression type, and RawMemoryOp collapsed load_u8/store_u8 into Load/Store, making later checks use the i32 ABI type as if it were the memory cell type.

## 問題

RawMemoryOp::FillBytes recorded an initialized byte range with the fill argument type. Additionally, RawMemoryHelper::LoadU8 and StoreU8 were lowered to the generic Load/Store operation, so initialized-move and owner checks had no enum-level way to distinguish byte memory cells from i32 ABI values.

## 影響

Safe public RegionToken/region_ptr provenance cannot prove u8 cells initialized after fill_u8/memset_u8. This is a compiler ResourceIR proof bug that can force tests or stdlib code to avoid valid checked bulk initialization paths.

## 修正方針

Keep byte memory operations explicit in ResourceIR: FillBytes records initialized byte ranges as u8 cells, and RawMemoryOp now has LoadU8/StoreU8 variants so exhaustive matches can choose the memory cell type independently from the i32 ABI value type. Add ResourceIR and stdlib memory_safety regression tests that require fill_u8 followed by load_u8 to pass, and that prevent FillBytes from proving a generic i32 cell initialized.

## 検証

- `cargo test -p nepl-core resource_ir_cell_check_fill_bytes --test resource_ir`
- `cargo test -p nepl-core resource_ir_cell_check_store_u8_initializes_u8_cells --test resource_ir`
- `cargo test -p nepl-core raw_memory --lib`
- `cargo check -p nepl-core`
- `trunk build`
- `node nodesrc/tests.js -i tests\\stdlib\\memory_safety.n.md --no-tree -o tmp\\agent1-memory-safety-raw-boundary-after.json -j 1 --dist web\\dist`
