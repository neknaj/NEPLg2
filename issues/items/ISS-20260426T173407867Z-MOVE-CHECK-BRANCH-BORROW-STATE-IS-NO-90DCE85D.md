---
id: ISS-20260426T173407867Z-MOVE-CHECK-BRANCH-BORROW-STATE-IS-NO-90DCE85D
title: "move check branch borrow state is not snapshotted"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-26
target: nepl-core/src/passes/move_check.rs
---

# ISS-20260426T173407867Z-MOVE-CHECK-BRANCH-BORROW-STATE-IS-NO-90DCE85D: move check branch borrow state is not snapshotted

## 概要

move_check が if/match/while の分岐探索で var state だけを履歴復元しており、borrow_stacks と borrow_counts の変更が探索外へ漏れる。branch 内の last-use release や reference assignment が別 branch/外側の lifetime 判定を汚染する。

## 対象

- `nepl-core/src/passes/move_check.rs`

## 根拠

- `MoveCheckContext::push_history` / `undo_history` は `var_stacks` の最上位状態だけを復元し、`borrow_stacks` と `borrow_counts` を復元しない。
- `visit_expr_with_escape` の `if` / `match` / `while` 処理は branch/body 探索後に `undo_history` だけで状態を戻す。
- `note_var_use` と `release_borrow_binding` は borrow lifetime release 時に `borrow_stacks` と `borrow_counts` を直接変更するため、branch 探索中の変更が外側の解析へ残る。

## 問題

move_check が if/match/while の分岐探索で var state だけを履歴復元しており、borrow_stacks と borrow_counts の変更が探索外へ漏れる。branch 内の last-use release や reference assignment が別 branch/外側の lifetime 判定を汚染する。

## 影響

borrow/lifetime 検査で false positive と false negative の両方が発生しうる。特に branch 内だけで参照を最後に使うケースや、branch 内で参照を保持するケースで move/borrow/drop 判定が実行経路と一致しない。

## 修正方針

branch/loop/match の探索単位で var_stacks, var_depth_stacks, borrow_stacks, borrow_counts をまとめて snapshot し、探索後に復元する。merge は継続する branch の Resource 状態だけを対象に行い、borrow lifetime が branch 探索から漏れないようにする。

## 検証

branch 内 last-use release と branch 内 retained borrow の compile/run 回帰テストを追加し、move_check と nodesrc compiler tests を通す。
