---
id: ISS-20260426T213057658Z-LLVM-CODEGEN-REACHES-UNBOUND-LOCALS--5CA65A55
title: "LLVM codegen reaches unbound locals after lowering"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-26
target: "nepl-core/src/codegen_llvm.rs, nepl-core/src/monomorphize.rs, stdlib/alloc/collections/adjacency_matrix.nepl"
---

# ISS-20260426T213057658Z-LLVM-CODEGEN-REACHES-UNBOUND-LOCALS--5CA65A55: LLVM codegen reaches unbound locals after lowering

## 概要

GitHub Actions run 24967172989 llvm-dual-stdlib reports D4102 unknown variable v reached llvm codegen for adjacency_matrix, bitset, bloom_filter, disjoint_set, fenwick, and segment_tree doctests; tests/compiler/move_effect also reports unknown variable u.

## 対象

- `nepl-core/src/codegen_llvm.rs, nepl-core/src/monomorphize.rs, stdlib/alloc/collections/adjacency_matrix.nepl`

## 根拠

- 未記入

## 問題

GitHub Actions run 24967172989 llvm-dual-stdlib reports D4102 unknown variable v reached llvm codegen for adjacency_matrix, bitset, bloom_filter, disjoint_set, fenwick, and segment_tree doctests; tests/compiler/move_effect also reports unknown variable u.

## 影響

A local binding can disappear between typecheck/monomorphize and LLVM codegen, which is a core correctness issue rather than a stdlib surface failure.

## 修正方針

Trace the HIR path that introduces these locals and ensure branch/match/destructure lowering preserves local definitions or reports a pre-codegen diagnostic when a binding is out of scope.

## 検証

Run llvm-dual focused tests for stdlib/alloc/collections/adjacency_matrix.nepl and tests/compiler/move_effect.n.md until no D4102 unknown variable diagnostics remain.
