---
id: ISS-20260428T132105486Z-RESOURCE-IR-LOWERING-COVERAGE-DOES-N-01BE2923
title: "Resource IR lowering coverage does not guard projection and borrow place completeness"
area: core
status: open
resolved: false
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

## 検証

Add resource_ir lowering tests that fail if AddrOf/Deref/field accessor expressions lower to Place::unknown or generic Expr-only operations, and run cargo test -p nepl-core --test resource_ir plus node nodesrc/issues.js check.
