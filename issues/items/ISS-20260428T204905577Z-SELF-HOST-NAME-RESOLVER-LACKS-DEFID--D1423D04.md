---
id: ISS-20260428T204905577Z-SELF-HOST-NAME-RESOLVER-LACKS-DEFID--D1423D04
title: "self-host name resolver lacks DefId and scope table"
area: selfhost
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-04-28
updated: 2026-04-28
target: stdlib/neplg2/core/resolve/name_resolver.nepl
---

# ISS-20260428T204905577Z-SELF-HOST-NAME-RESOLVER-LACKS-DEFID--D1423D04: self-host name resolver lacks DefId and scope table

## 概要

resolve/name_resolver.nepl still exposes only a marker API and has no DefId, binding record, or scope table. Later hoist/import/check stages would each need to invent their own string-to-definition representation.

## 対象

- `stdlib/neplg2/core/resolve/name_resolver.nepl`

## 根拠

- `stdlib/neplg2/core/resolve/name_resolver.nepl` は `selfhost_name_resolver_stage0` だけを公開しており、scope table や定義 id の所有境界がなかった。
- HIR / check / diagnostic が後で定義を参照するには、raw string ではなく安定した `DefId` が必要になる。
- shadowing の基本動作を 1 箇所に固定しないと、import / hoist / local scope の各 stage が独自の検索規則を持つ危険がある。

## 問題

resolve/name_resolver.nepl still exposes only a marker API and has no DefId, binding record, or scope table. Later hoist/import/check stages would each need to invent their own string-to-definition representation.

## 影響

Name resolution cannot become a stable pass boundary, and HIR/type stages cannot refer to definitions through stable ids. This keeps RV-STDLIB-008 blocked and risks duplicating raw string lookup logic across self-host stages.

## 修正方針

`SelfhostDefId`、`SelfhostDefKind`、`SelfhostNameBinding`、`SelfhostNameScope` を追加しました。

scope は binding table を所有し、`selfhost_name_scope_add_binding` で追加位置を安定した `DefId` として割り当てます。`selfhost_name_scope_get` は `DefId` から O(1) で record を返し、`selfhost_name_scope_find` は末尾から線形探索して同一 scope 内の後勝ち shadowing を固定します。

現段階では親 scope、import、hoist には踏み込まず、後続 stage が共有できる最小の名前解決データ境界を作りました。

## 検証

- `trunk build`: pass
- `node nodesrc\tests.js -i stdlib\neplg2\core\resolve\name_resolver.nepl --no-tree -o tmp\selfhost-name-scope-bindings.json -j 1`: total=1 passed=1
- `node nodesrc\issues.js check`: pass, files=317
