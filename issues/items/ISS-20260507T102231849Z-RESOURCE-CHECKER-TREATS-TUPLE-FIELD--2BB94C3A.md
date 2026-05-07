---
id: ISS-20260507T102231849Z-RESOURCE-CHECKER-TREATS-TUPLE-FIELD--2BB94C3A
title: "Resource checker treats tuple field extraction as moving whole tuple in overload fixture"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-07
updated: 2026-05-07
target: "tests/compiler/overload.n.md, nepl-core/src/resource"
---

# ISS-20260507T102231849Z-RESOURCE-CHECKER-TREATS-TUPLE-FIELD--2BB94C3A: Resource checker treats tuple field extraction as moving whole tuple in overload fixture

## 概要

`tests/compiler/overload.n.md::overload_pair_field_from_generic_result_keeps_tuple_type` extracts two owned `Vec` fields from the same `Tuple` using `field::get`. Current ResourceIR reports `resource.cell.moved` on the second read of `parts`, so focused overload doctests fail before unrelated stdlib API fixture updates can be validated.

## 対象

- `tests/compiler/overload.n.md, nepl-core/src/resource`

## 根拠

- `let evens <Vec<i32>> get parts 0;` succeeds by moving one owner field out of `parts`.
- `let rest <Vec<i32>> get parts 1;` then reports `resource.cell.moved` on `parts`.
- This means the checker currently treats the first field extraction as a move of the whole aggregate, not as a per-field move or an explicit all-field destructuring operation.

## 問題

Owned aggregate field extraction cannot be relied on for tuples carrying multiple owners. The language must either support field-level move state for aggregates or require a single statically checked destructuring operation that moves all owner fields exactly once.

## 影響

Selfhost code and stdlib helpers that return aggregates containing multiple owners cannot be validated precisely. This weakens type/resource regression coverage for ownership-preserving APIs and can force unnatural workaround code around aggregate returns.

## 修正方針

Review ResourceIR aggregate field move semantics for `Tuple` and struct owners. It should either support independent owned field extraction with per-field move state, or introduce/use a destructuring operation that statically moves each field once. Add a focused regression for extracting both owner fields from one aggregate.

## 検証

- `node nodesrc/tests.js -i tests/compiler/overload.n.md --no-tree -o tmp/overload-after-stack-primary-observers.json -j 1 --dist web/dist`: total=45, passed=44, failed=1 (`resource.cell.moved` in `overload_pair_field_from_generic_result_keeps_tuple_type`)

## 2026-05-07 対応結果

- Resource IR lowering の aggregate field selector 判定を enum 化し、数値 index と文字列 field 名を同じ projection helper で扱うようにした。
- `core/field::get` / `get_ref` と `get_field` / `get_field_ref` intrinsic の lowering が `get parts 0` を通常 call 引数ではなく `%parts.tuple0` の field read / borrow として下げるようにした。
- Resource coverage gate も同じ selector 判定へ揃え、HIR coverage が数値 tuple selector を direct call と誤分類しないようにした。
- `resource_ir_cell_check_moves_numeric_tuple_fields_independently` を追加し、`parts.tuple0` と `parts.tuple1` の独立 move、Resource IR dump 上の明示 projection、full Resource IR pipeline 通過を固定した。
- commit 前に `origin/main` の `refactor(stdlib): split integer common helpers` を取り込み、issue index を再生成した。
- focused verification:
  - `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_moves_numeric_tuple_fields_independently -- --nocapture`
  - `trunk build --release`
  - `node nodesrc/run_doctest.js -i tests/compiler/overload.n.md -n 10 --dist web/dist`
  - `node nodesrc/tests.js -i tests/compiler/overload.n.md --no-tree -o tmp/overload-resource-tuple-field-move-after-sync.json -j 1 --dist web/dist`
