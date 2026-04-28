---
id: ISS-20260428T132105486Z-RESOURCE-IR-LOWERING-COVERAGE-DOES-N-01BE2923
title: "Resource IR lowering coverage does not guard projection and borrow place completeness"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/resource/lower.rs, nepl-core/src/resource/coverage.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260428T132105486Z-RESOURCE-IR-LOWERING-COVERAGE-DOES-N-01BE2923: Resource IR lowering coverage does not guard projection and borrow place completeness

## 概要

Resource IR lowering still maps non-variable addressable expressions to Place::unknown, and Deref remains a generic Expr without a resource place transition. The lowering coverage only compares direct calls, indirect calls, function values, and raw memory op counts, so missing field/deref/borrow/projection lowering can produce no lowering diagnostic.

## 対象

- `nepl-core/src/resource/lower.rs`
- `nepl-core/src/resource/coverage.rs`
- `nepl-core/tests/resource_ir.rs`
- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 3 / Stage 4

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 3 は field projection、`MemPtr` projection、storage owner、raw load/store を Resource IR op に下げる計画である。
- `nepl-core/src/resource/lower.rs` の `place_from_expr_skeleton` は `HirExprKind::Var` 以外を `Place::unknown(expr.ty)` にする。
- `HirExprKind::Deref` は inner を下げた後に `ResourceExprKind::Deref` を出すだけで、`PlaceProjection::Deref` や storage/provenance transition を作らない。
- `nepl-core/src/resource/coverage.rs` は direct call、indirect call、function value、raw memory op count だけを比較し、unknown place、borrow、construct、read/move/assign、projection-producing expression の coverage を診断しない。

## 問題

Resource IR lowering still maps non-variable addressable expressions to Place::unknown, and Deref remains a generic Expr without a resource place transition. The lowering coverage only compares direct calls, indirect calls, function values, and raw memory op counts, so missing field/deref/borrow/projection lowering can produce no lowering diagnostic.

## 影響

Resource checks can remain shadow-clean while the Resource IR has lost the place identity needed for borrow, owner, initialized cell, and raw provenance enforcement. Switching Stage 4 checks from shadow to authoritative in this state would either miss memory-safety cases or require falling back to old move_check HIR summaries.

## 修正方針

Extend Resource IR lowering to represent addressable field, tuple, enum payload, deref, and storage-offset projections as Place projections, then extend ResourceLoweringCoverage with counts or diagnostics for unknown places, borrow ops, construct ops, read/move/assign ops, and projection-producing expressions.

## 修正内容

- `AddrOf` lowering は addressable expression の `Place` skeleton を優先し、未知 place に落ちる場合だけ inner expression lowering の出力 temp を borrow source にするようにした。
- `Deref` lowering は generic `Expr` のみではなく、source place に `PlaceProjection::Deref` を付けた `Read` を生成してから deref expr output を保持するようにした。
- `place_from_expr_skeleton` は nested deref と `add(base, offset)` の storage offset projection を扱い、Resource IR 上の pointer / storage projection を失わないようにした。
- `ResourceLoweringCoverage` は construct / declare / read / move / assign / borrow / drop / deref projection / unknown place を比較し、Resource IR 側に `Place::unknown` が残った場合は operation 名付きで `UnknownPlace` diagnostic を出すようにした。
- `resource_ir_lowering_coverage_guards_borrow_and_deref_places` を追加し、borrow source と deref read source が `PlaceProjection::Deref` を持つこと、壊れた Resource IR では unknown place diagnostic と count mismatch が出ることを固定した。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_lowering_coverage_guards_borrow_and_deref_places -- --nocapture`
- `cargo test -p nepl-core --test resource_ir -- --nocapture`
- commit 前に `trunk build`、`node nodesrc/issues.js check`、`rustfmt --check`、`git diff --check` を実行する。
