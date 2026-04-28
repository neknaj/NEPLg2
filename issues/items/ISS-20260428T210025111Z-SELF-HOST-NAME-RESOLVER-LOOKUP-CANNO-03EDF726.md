---
id: ISS-20260428T210025111Z-SELF-HOST-NAME-RESOLVER-LOOKUP-CANNO-03EDF726
title: "self-host name resolver lookup cannot filter definition kind"
area: selfhost
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-04-28
updated: 2026-04-28
target: stdlib/neplg2/core/resolve/name_resolver.nepl
---

# ISS-20260428T210025111Z-SELF-HOST-NAME-RESOLVER-LOOKUP-CANNO-03EDF726: self-host name resolver lookup cannot filter definition kind

## 概要

selfhost_name_scope_find returns the latest binding by name only. A later type declaration and value declaration with the same text would shadow each other even when the caller is resolving only value names or only type names.

## 対象

- `stdlib/neplg2/core/resolve/name_resolver.nepl`

## 根拠

- `selfhost_name_scope_find` は同名 binding の末尾から最初に見つかったものを返すため、名前だけの shadowing には対応できる。
- しかし `SelfhostDefKind` は value / type / trait などの名前空間を分けるために存在するにもかかわらず、検索 API では使われていなかった。
- self-host checker / import resolver が kind ごとの lookup を必要とするたびに独自探索を実装すると、shadowing 規則が分散する。

## 問題

selfhost_name_scope_find returns the latest binding by name only. A later type declaration and value declaration with the same text would shadow each other even when the caller is resolving only value names or only type names.

## 影響

Self-host checker/import stages need value/type/trait namespace separation. Without a DefKind-filtered lookup, later stages would either duplicate scope traversal or accidentally resolve the wrong namespace.

## 修正方針

`selfhost_name_scope_find_kind_loop` と `selfhost_name_scope_find_kind` を追加しました。

既存の `selfhost_name_scope_find` は「名前だけで最新 binding」を返す API として維持し、名前空間を分ける caller は `find_kind` を使う形にしました。`find_kind` は末尾から探索し、名前と `SelfhostDefKind` の両方が一致する binding だけを返します。

同名 `Local` / `Struct` binding を追加し、名前だけ lookup では最新の `Struct` を返し、`find_kind Local` では古い `Local` を返し、存在しない `Builtin` は `None` になることを doctest で固定しました。

## 検証

- `node nodesrc\tests.js -i stdlib\neplg2\core\resolve\name_resolver.nepl --no-tree -o tmp\selfhost-name-kind-lookup.json -j 1`: total=2 passed=2
- `node nodesrc\issues.js check`: pass, files=318
