---
id: ISS-20260507T161930297Z-SELFHOST-HIR-EXPRESSIONS-STORE-KIND--54E75EE3
title: "Selfhost HIR expressions store kind-specific fields in a flat record"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-07
updated: 2026-05-08
target: "stdlib/neplg2/core/hir/hir.nepl, nodesrc/test_selfhost_hir_expr_payload.js"
---

# ISS-20260507T161930297Z-SELFHOST-HIR-EXPRESSIONS-STORE-KIND--54E75EE3: Selfhost HIR expressions store kind-specific fields in a flat record

## 概要

SelfhostHirExpr stores first_child, child_count, name, int_value, and bool_value for every expression kind, so callers can read fields that are meaningless for the actual variant.

## 対象

- `stdlib/neplg2/core/hir/hir.nepl, nodesrc/test_selfhost_hir_expr_payload.js`

## 根拠

- 未記入

## 問題

SelfhostHirExpr stores first_child, child_count, name, int_value, and bool_value for every expression kind, so callers can read fields that are meaningless for the actual variant.

## 影響

HIR lowering and later type/resource checks cannot rely on match exhaustiveness to prove which payload is valid. Placeholder fields such as empty names or zero literal values can mask missing lowering branches and weaken typed self-host IR guarantees.

## 修正方針

Move expression-specific data into a SelfhostHirExprPayload enum. Keep only common ty/span fields on SelfhostHirExpr, expose kind and child range through match-based accessors, and replace flat constructors with variant-specific constructors.

## 検証

Add a source policy rejecting the flat payload fields and flat constructors. Run focused HIR doctests, issue check, and source policy regressions.

## 2026-05-08 解決

`SelfhostHirExpr` から `kind` / `first_child` / `child_count` / `name` / `int_value` / `bool_value` の flat fields を削除し、common な `ty` / `span` と `SelfhostHirExprPayload` だけを持つ record に変更した。

`SelfhostHirExprPayload` は `Error` / `Unit` / `BoolLiteral` / `I32Literal` / `StrLiteral` / `Var` / `Call` / `Block` / `If` に分かれ、Call 専用 payload は `SelfhostHirCallExpr` が `name` と `args` を所有する。child range は `Call` / `Block` / `If` payload を match した場合だけ取得できる。

`selfhost_hir_expr_kind` と `selfhost_hir_expr_child_range` は `&SelfhostHirExpr` を受け、borrowed payload を match して分類と child range を返す。flat constructor の `selfhost_hir_expr_leaf` / `selfhost_hir_expr_with_children` は削除し、variant-specific constructor に置き換えた。

回帰防止として `nodesrc/test_selfhost_hir_expr_payload.js` を追加し、次を拒否する。

- `SelfhostHirExpr` に kind-specific fields が戻る実装
- `selfhost_hir_expr_leaf` / `selfhost_hir_expr_with_children`
- `SelfhostHirExprKind` を渡す flat constructor call
- accessor が payload match を通さず flat fields を読む実装

検証:

- `node nodesrc/test_selfhost_hir_expr_payload.js`: passed
- `node nodesrc/tests.js -i stdlib/neplg2/core/hir/hir.nepl --no-tree -o tmp/agent1-selfhost-hir-expr-payload.json -j 1 --dist web/dist`: total=3, passed=3
