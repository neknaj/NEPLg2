---
id: ISS-20260426T221523543Z-AGGREGATE-FIELD-TEMPORARY-BORROWS-DO-8E71F5EA
title: "Aggregate field temporary borrows do not overlap"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-27
target: "nepl-core/src/passes/move_check.rs, nepl-core/tests/move_check.rs, tests/compiler/move_check.n.md"
---

# ISS-20260426T221523543Z-AGGREGATE-FIELD-TEMPORARY-BORROWS-DO-8E71F5EA: Aggregate field temporary borrows do not overlap

## 概要

move_check visits struct and tuple aggregate items independently and does not retain each item's borrow origins while later items are checked, so RefPair &mut x &x and Tuple: &mut x; &x compile successfully.

## 対象

- `nepl-core/src/passes/move_check.rs, nepl-core/tests/move_check.rs, tests/compiler/move_check.n.md`

## 根拠

- `HirExprKind::StructConstruct` / `TupleConstruct` は field/item ごとに `visit_expr_with_escape` を呼び、返ってきた `ExprBorrow` を `result_borrows` に集めるだけで、次の field/item の検査中に active borrow として保持していない。
- そのため先に評価した `&mut x` が次の field/item の `&x` 検査時に `BorrowedUnique` として見えない。
- 最小再現として `struct RefPair: a <&mut LocalToken>; b <&LocalToken>` に対する `let p <RefPair> RefPair &mut x &x` が `cargo run -q -p nepl-cli -- --check --target core` で `Check successful` になる。
- `let p Tuple: &mut x; &x` も同じく `Check successful` になる。

## 問題

move_check visits struct and tuple aggregate items independently and does not retain each item's borrow origins while later items are checked, so RefPair &mut x &x and Tuple: &mut x; &x compile successfully.

## 影響

Borrow/lifetime checking is unsound for aggregate construction: a single constructed value can hold overlapping shared and unique references to the same owner.

## 修正方針

Retain borrow origins from aggregate fields/items during aggregate construction until all fields/items have been checked, then return those origins to the enclosing expression so stored aggregates keep the borrows alive.

## 検証

Add compile_fail tests for struct and tuple aggregate constructors containing overlapping &mut/& borrows, plus focused move_check Rust and n.md tests.

## 解決

- `visit_aggregate_items_with_escape` を追加し、struct field / tuple item から得た borrow origin を aggregate 構築中の一時 borrow として retain するようにした。
- 全 field/item の検査後に構築中 temporary は release し、aggregate 値が外側へ返す borrow origin は従来どおり `result_borrows` として返すようにした。
- struct と tuple の `&mut x` / `&x` overlap を compile_fail として追加した。

## 修正後検証

- `cargo fmt --all --check`: pass
- `cargo test -p nepl-core --test move_check -- --nocapture`: 45/45 passed
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/move_check.n.md -i tests/compiler/move_effect.n.md --no-tree -o tmp/aggregate-temporary-borrow-tests.json -j 1`: 73/73 passed
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass
