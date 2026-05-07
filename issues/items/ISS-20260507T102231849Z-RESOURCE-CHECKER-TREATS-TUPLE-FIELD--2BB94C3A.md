---
id: ISS-20260507T102231849Z-RESOURCE-CHECKER-TREATS-TUPLE-FIELD--2BB94C3A
title: "Resource checker treats tuple field extraction as moving whole tuple in overload fixture"
area: core
status: open
resolved: false
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
