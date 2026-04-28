---
id: ISS-20260428T173213551Z-RESOURCE-CELLSTATE-CHECKER-CONSUMES--DD20A3D7
title: "Resource CellState checker consumes raw memory helper arguments before raw operations"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-29
target: "nepl-core/src/resource/initialized.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260428T173213551Z-RESOURCE-CELLSTATE-CHECKER-CONSUMES--DD20A3D7: Resource CellState checker consumes raw memory helper arguments before raw operations

## 概要

Resource IR lowering emits a generic Call followed by a RawMemory op for direct raw memory helpers. The CellState checker consumes Call arguments first, so a non-Copy value passed to store is moved before RawMemory::Store can initialize the pointed cell.

## 対象

- `nepl-core/src/resource/initialized.rs`
- `nepl-core/tests/resource_ir.rs`
- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 は initialized / moved cell state を Resource IR 側へ移す計画である。
- Resource IR lowering は direct raw memory helper call に対して、effect / call target の記録として `ResourceOp::Call` を出し、raw memory semantics として `ResourceOp::RawMemory` も続けて出す。
- CellState checker が両方を同じ所有権操作として扱うと、`store<T>(ptr, value)` の `value` が `CallArgument` で先に moved になり、直後の `RawMemory::Store` が `ptr.*` を initialized にできない。
- `RawMemoryLoadCell` を compiler gate に一時的に入れた調査で、`helper-returned slot` 系の一部だけでなく、store 済みの raw slot を uninitialized と見る false D3100 が発生することを確認した。

## 問題

Resource IR lowering emits a generic Call followed by a RawMemory op for direct raw memory helpers. The CellState checker consumes Call arguments first, so a non-Copy value passed to store is moved before RawMemory::Store can initialize the pointed cell.

## 影響

RawMemoryLoadCell enforcement sees later loads as uninitialized even when the program stored the value through the same raw address. This creates false D3100 diagnostics and blocks Stage 4 from making raw load cell checks authoritative.

## 修正方針

Treat direct raw-memory helper Call ops as semantic placeholders for CellState and let the following RawMemory op own argument consumption and output initialization. Add a regression with Call+RawMemory::Store followed by RawMemory::Load for a non-Copy value.

## 修正内容

- CellState checker は `EffectOp::InternalAlloc` / `EffectOp::UnsafeMemory` の direct `ResourceOp::Call` を initialized/moved state の消費対象にしないようにした。
- raw memory helper の引数消費、raw cell initialized/moved transition、output initialization は、直後の `ResourceOp::RawMemory` に一本化した。
- `resource_ir_cell_check_raw_memory_call_does_not_consume_store_value_twice` を追加し、generic `Call` + `RawMemory::Store` + `RawMemory::Load` の並びで非Copy store valueが二重消費されないことを固定した。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_raw_memory_call_does_not_consume_store_value_twice -- --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 70 passed
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc\tests.js -i tests\compiler\move_effect.n.md --no-tree -o tmp\resource-cellstate-raw-call-move-effect.json -j 1`: total=110, passed=110
- `node nodesrc\tests.js -i tests\compiler\move_check.n.md --no-tree -o tmp\resource-cellstate-raw-call-move-check.json -j 1`: total=52, passed=52
