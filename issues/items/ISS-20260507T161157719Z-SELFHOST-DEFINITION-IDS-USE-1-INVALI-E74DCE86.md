---
id: ISS-20260507T161157719Z-SELFHOST-DEFINITION-IDS-USE-1-INVALI-E74DCE86
title: "Selfhost definition IDs use -1 invalid sentinel"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-07
updated: 2026-05-08
target: "stdlib/neplg2/core/resolve/name_resolver.nepl, nodesrc/test_selfhost_def_id_absence.js"
---

# ISS-20260507T161157719Z-SELFHOST-DEFINITION-IDS-USE-1-INVALI-E74DCE86: Selfhost definition IDs use -1 invalid sentinel

## 概要

SelfhostDefId exposes selfhost_def_id_invalid and SelfhostNameBinding stores an unset definition reference as index -1 before insertion into a scope.

## 対象

- `stdlib/neplg2/core/resolve/name_resolver.nepl, nodesrc/test_selfhost_def_id_absence.js`

## 根拠

- 未記入

## 問題

SelfhostDefId exposes selfhost_def_id_invalid and SelfhostNameBinding stores an unset definition reference as index -1 before insertion into a scope.

## 影響

Resolver and later HIR/type stages can accidentally treat an unresolved binding as a valid definition table index. This keeps absence in an i32 payload and bypasses Option match coverage.

## 修正方針

Remove invalid DefId construction. Store optional binding definition IDs as Option<SelfhostDefId>, construct pending bindings with None, and assign Some(def_id) when the scope table allocates a stable definition ID.

## 検証

Add a source policy rejecting selfhost_def_id_invalid and selfhost_def_id_new -1. Run focused name_resolver doctests, issue check, and source policy regressions.

## 2026-05-08 解決

`selfhost_def_id_invalid` を削除し、未割り当て定義 id は `Option<SelfhostDefId>` で表すようにした。`selfhost_def_id_pending` は `None`、`selfhost_def_id_assigned` は stable scope table id の `Some` を返す。

`SelfhostNameBinding.def_id` は `Option<SelfhostDefId>` になった。scope 追加前の binding は `selfhost_name_binding_pending` で `None` を持ち、`selfhost_name_scope_add_binding` が scope table の位置から stable `SelfhostDefId` を割り当てた時点でだけ `Some(def_id)` を保存する。追加時には入力 binding の def_id を信用せず、scope 側で割り当て直す。

回帰防止として `nodesrc/test_selfhost_def_id_absence.js` を追加し、次を拒否する。

- `fn selfhost_def_id_invalid`
- `selfhost_def_id_new -1`
- `SelfhostNameBinding.def_id` が非 Option に戻る実装
- scope insertion が pre-insertion binding の def_id を信用する実装

検証:

- `node nodesrc/test_selfhost_def_id_absence.js`: passed
- `node nodesrc/tests.js -i stdlib/neplg2/core/resolve/name_resolver.nepl --no-tree -o tmp/agent1-selfhost-def-id-absence.json -j 1 --dist web/dist`: total=2, passed=2
