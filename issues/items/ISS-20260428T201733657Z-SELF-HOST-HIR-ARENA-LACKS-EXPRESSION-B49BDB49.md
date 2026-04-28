---
id: ISS-20260428T201733657Z-SELF-HOST-HIR-ARENA-LACKS-EXPRESSION-B49BDB49
title: "self-host HIR arena lacks expression child range API"
area: selfhost
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-04-28
updated: 2026-04-28
target: stdlib/neplg2/core/hir/hir.nepl
---

# ISS-20260428T201733657Z-SELF-HOST-HIR-ARENA-LACKS-EXPRESSION-B49BDB49: self-host HIR arena lacks expression child range API

## 概要

HIR expression record has first_child / child_count fields and the module owns expr_children, but there is no public API to allocate a contiguous child range or read a child id by range/index. Lowering for Call / Block / If would need to write raw offsets by hand, which makes the arena invariant unclear.

## 対象

- `stdlib/neplg2/core/hir/hir.nepl`

## 根拠

- `SelfhostHirExpr` は `first_child` / `child_count` を持つ。
- `SelfhostHirModule` は `expr_children` table を所有する。
- 修正前は `expr_children` へ child id 列を追加する API と、range から child id を読む API がなかった。

## 問題

HIR expression record has first_child / child_count fields and the module owns expr_children, but there is no public API to allocate a contiguous child range or read a child id by range/index. Lowering for Call / Block / If would need to write raw offsets by hand, which makes the arena invariant unclear.

## 影響

S4 HIR lowering cannot construct non-leaf expressions through the stable module boundary. The current design invites ad hoc offset management in later passes and leaves RV-STDLIB-008 progress blocked at leaf expressions.

## 修正方針

`SelfhostHirChildRange` と `SelfhostHirModuleChildRangeAlloc` を追加し、child id 列を module の `expr_children` table へコピーして typed range を返す API を追加しました。

あわせて、child range から expression record を作る `selfhost_hir_expr_with_children`、expression から child range を取り出す `selfhost_hir_expr_child_range`、range + index を bounds check して child id を返す `selfhost_hir_module_get_child` を追加しました。これにより、後続 lowering は raw offset を直接組み立てずに `Call` / `Block` / `If` などの non-leaf expression を作れます。

## 検証

- `node nodesrc\tests.js -i stdlib\neplg2\core\hir\hir.nepl --no-tree -o tmp\selfhost-hir-child-ranges.json -j 1`: total=2 passed=2
