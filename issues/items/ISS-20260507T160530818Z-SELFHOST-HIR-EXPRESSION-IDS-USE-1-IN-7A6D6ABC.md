---
id: ISS-20260507T160530818Z-SELFHOST-HIR-EXPRESSION-IDS-USE-1-IN-7A6D6ABC
title: "Selfhost HIR expression IDs use -1 invalid sentinel"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-07
updated: 2026-05-08
target: "stdlib/neplg2/core/hir/hir.nepl, nodesrc/test_selfhost_hir_expr_id_absence.js"
---

# ISS-20260507T160530818Z-SELFHOST-HIR-EXPRESSION-IDS-USE-1-IN-7A6D6ABC: Selfhost HIR expression IDs use -1 invalid sentinel

## 概要

SelfhostHirExprId exposes selfhost_hir_expr_id_invalid, representing an unset expression reference as index -1 instead of typed absence.

## 対象

- `stdlib/neplg2/core/hir/hir.nepl, nodesrc/test_selfhost_hir_expr_id_absence.js`

## 根拠

- 未記入

## 問題

SelfhostHirExprId exposes selfhost_hir_expr_id_invalid, representing an unset expression reference as index -1 instead of typed absence.

## 影響

HIR lowering and later static checks can accidentally pass an unset expression ID as an ordinary table index. This weakens the self-host typed model policy that absence must be represented by Option or enum payloads instead of numeric sentinels.

## 修正方針

Remove invalid expression ID construction and represent pending or unset expression references as Option<SelfhostHirExprId>. Keep SelfhostHirExprId as a stable expression table index only.

## 検証

Add a source policy rejecting selfhost_hir_expr_id_invalid and selfhost_hir_expr_id_new -1. Run focused HIR doctests, issue check, and source policy regressions.

## 2026-05-08 解決

`selfhost_hir_expr_id_invalid` を削除し、未設定 expression id は `Option<SelfhostHirExprId>` で表すようにした。`selfhost_hir_expr_id_pending` は `None`、`selfhost_hir_expr_id_assigned` は stable arena id の `Some` を返す。

`SelfhostHirExprId` 自体は HIR module 内 expression table の index だけを表す型として残し、未設定状態を `-1` payload として通常の id に混ぜない。`selfhost_hir_stage0` では pending / assigned の両方を Option match で検査し、typed absence helper が current compiler で実行可能であることを固定した。

回帰防止として `nodesrc/test_selfhost_hir_expr_id_absence.js` を追加し、次を拒否する。

- `fn selfhost_hir_expr_id_invalid`
- `selfhost_hir_expr_id_new -1`
- pending state が invalid id payload を返す実装

検証:

- `node nodesrc/test_selfhost_hir_expr_id_absence.js`: passed
- `node nodesrc/tests.js -i stdlib/neplg2/core/hir/hir.nepl --no-tree -o tmp/agent1-selfhost-hir-expr-id-absence.json -j 1 --dist web/dist`: total=3, passed=3
