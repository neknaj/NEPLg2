---
id: ISS-20260428T171533230Z-RESOURCE-IR-LOWERING-EMITS-UNKNOWN-R-ACE2EF90
title: "Resource IR lowering emits unknown return place for unit while expressions"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-29
target: "nepl-core/src/resource/lower.rs, nepl-core/src/resource/initialized.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260428T171533230Z-RESOURCE-IR-LOWERING-EMITS-UNKNOWN-R-ACE2EF90: Resource IR lowering emits unknown return place for unit while expressions

## 概要

Resource IR lowering lowers HirExprKind::While to ResourceOp::Loop but returns Place::unknown(expr.ty). When a unit-return function ends with a while expression, the Stage 4 lowering coverage gate reports D3101 unknown return place even though the HIR is valid.

## 対象

- `nepl-core/src/resource/lower.rs`
- `nepl-core/src/resource/initialized.rs`
- `nepl-core/tests/resource_ir.rs`
- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 3 / Stage 4

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 3 は HIR から Resource IR への lowering completeness を前提にしている。
- Stage 4 の D3101 coverage gate は `Place::unknown` を compiler error として扱うため、Resource IR が valid HIR の unit expression を concrete place に落とせない場合、検査の不備ではなく lowering の表現不足としてCIを止める。
- 2026-04-28 の main CI では `tests/compiler/list_dot_map.n.md`、`tests/stdlib/adjacency_matrix_collections.n.md`、`tests/stdlib/binary_heap_collections.n.md` などで、unit-return function の最後にある `while` が `return unknown:t0` になり D3101 を出していた。

## 問題

Resource IR lowering lowers HirExprKind::While to ResourceOp::Loop but returns Place::unknown(expr.ty). When a unit-return function ends with a while expression, the Stage 4 lowering coverage gate reports D3101 unknown return place even though the HIR is valid.

## 影響

Main CI can fail on valid stdlib and compiler doctests after Resource IR coverage enforcement. The false positive blocks authoritative static checks and hides real lowering-completeness regressions behind noisy unit-return diagnostics.

## 修正方針

Make while lowering produce a concrete ResourceExprKind::Loop temporary for the expression result, mark loop expression outputs initialized in the CellState checker, and add regression coverage that unit while returns are concrete and D3101-clean.

## 修正内容

- `HirExprKind::While` lowering は `ResourceOp::Loop` を生成した後、`Place::unknown(expr.ty)` ではなく `ResourceExprKind::Loop` の temporary を返すようにした。
- CellState checker は `ResourceExprKind::Loop` の output を initialized として扱い、unit-return の `while` が未初期化戻り値として誤診断されないようにした。
- `resource_ir_lowering_returns_concrete_unit_place_for_while` を追加し、coverage diagnostic が空であること、initialized/move report が空であること、return value と loop expr output が unknown ではないことを固定した。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_lowering_returns_concrete_unit_place_for_while -- --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 69 passed
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc\tests.js -i tests\compiler\list_dot_map.n.md --no-tree -o tmp\resource-return-coverage-list-dot-map.json -j 1`: total=4, passed=4
- `node nodesrc\tests.js -i tests\stdlib\adjacency_matrix_collections.n.md --no-tree -o tmp\resource-return-coverage-adjacency-matrix.json -j 1`: total=2, passed=2
- `node nodesrc\tests.js -i tests\stdlib\binary_heap_collections.n.md --no-tree -o tmp\resource-return-coverage-binary-heap.json -j 1`: total=3, passed=3
