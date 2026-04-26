---
id: ISS-20260426T215554373Z-COPY-VALUE-RETAINED-BORROWS-SKIP-SHA-8309070D
title: "Copy value retained borrows skip shared/unique exclusivity checks"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-27
target: "nepl-core/src/passes/move_check.rs, nepl-core/tests/move_check.rs, tests/compiler/move_check.n.md"
---

# ISS-20260426T215554373Z-COPY-VALUE-RETAINED-BORROWS-SKIP-SHA-8309070D: Copy value retained borrows skip shared/unique exclusivity checks

## 概要

move_check skips temporary borrow validation for Copy values, so a retained &mut i32 can coexist with a retained &i32 or another &mut i32 without a diagnostic.

## 対象

- `nepl-core/src/passes/move_check.rs, nepl-core/tests/move_check.rs, tests/compiler/move_check.n.md`

## 根拠

- `MoveCheckContext::check_temporary_borrow` は対象式の型が `Copy` の場合に即 return する。
- そのため `let u <&mut i32> &mut x` で `x` が unique borrowed になった後でも、`let s <&i32> &x` の borrow creation 検査が実行されない。
- `retain_borrow_binding` は conflict diagnostic を出さず、既に `BorrowedUnique` の source に shared binding を保持する変数を作れてしまう。
- 最小再現は `cargo run -q -p nepl-cli -- --check --target core` に stdin で `let u <&mut i32> &mut x; let s <&i32> &x; let keep <&mut i32> u` を渡すと `Check successful` になる。
- 同じ最小再現を `nodesrc/run_test.js` の `compile_fail` として実行すると `expected compile_fail, but compiled successfully` になる。

## 問題

move_check skips temporary borrow validation for Copy values, so a retained &mut i32 can coexist with a retained &i32 or another &mut i32 without a diagnostic.

## 影響

Borrow/lifetime checks become unsound for Copy locals: Copy values may still be read while shared-borrowed, but retained references must preserve shared/unique exclusivity and currently do not.

## 修正方針

Separate Copy owner value reuse from borrow creation checks. Taking a borrow of a Copy value should still reject conflicts with live unique/shared borrows, while value copying from a shared borrow remains allowed.

## 検証

Add focused Copy-value reference alias compile_fail tests and run cargo test -p nepl-core --test move_check plus nodesrc move_check doctests.

## 解決

- `MoveCheckContext::check_temporary_borrow` から `Copy` 型での早期 return を削除し、borrow creation 時の shared/unique 排他検査を型の Copy 性とは独立して実行するようにした。
- `check_use` 側の挙動は維持し、shared borrow 中の `Copy` 値を通常値としてコピーして使うケースは引き続き許可した。
- `&mut i32` 生存中の `&i32` 作成、`&i32` 生存中の `&mut i32` 作成を compile_fail として追加し、shared borrow 中の `Copy` 値コピーが通ることも回帰テストに追加した。

## 修正後検証

- `cargo fmt --all --check`: pass
- `cargo test -p nepl-core --test move_check -- --nocapture`: 41/41 passed
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/move_check.n.md -i tests/compiler/move_effect.n.md --no-tree -o tmp/copy-borrow-exclusivity-tests.json -j 1`: 69/69 passed
- `node nodesrc/issues.js check`: pass
