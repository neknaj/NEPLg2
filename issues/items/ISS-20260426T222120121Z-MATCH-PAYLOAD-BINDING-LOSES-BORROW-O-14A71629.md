---
id: ISS-20260426T222120121Z-MATCH-PAYLOAD-BINDING-LOSES-BORROW-O-14A71629
title: "Match payload binding loses borrow origin"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-27
target: "nepl-core/src/passes/move_check.rs, nepl-core/tests/move_check.rs, tests/compiler/move_check.n.md"
---

# ISS-20260426T222120121Z-MATCH-PAYLOAD-BINDING-LOSES-BORROW-O-14A71629: Match payload binding loses borrow origin

## 概要

move_check declares enum match payload bindings without the scrutinee's borrow origins, so a reference payload extracted from RefOpt::Some &x no longer keeps x borrowed.

## 対象

- `nepl-core/src/passes/move_check.rs, nepl-core/tests/move_check.rs, tests/compiler/move_check.n.md`

## 根拠

- `HirExprKind::Match` は `visit_expr(scrutinee, ...)` の戻り値を捨てており、scrutinee に含まれる borrow origin を arm へ渡していない。
- arm の `bind_local` は `ctx.declare_var(bind.clone())` で宣言されるため、payload binding の `borrow_stacks` は空になる。
- さらに `visit_expr` の iterative fast path は `escape_depth == None` の式で常に `Vec::new()` を返すため、`RefOpt` のような reference を含む型の `Var(e)` からも borrow origin が返らない。
- 最小再現として `RefOpt::Some &x` を `match` し、`Some r` arm で `let y <LocalToken> x; let keep <&LocalToken> r` としても `cargo run -q -p nepl-cli -- --check --target core` が `Check successful` になる。

## 問題

move_check declares enum match payload bindings without the scrutinee's borrow origins, so a reference payload extracted from RefOpt::Some &x no longer keeps x borrowed.

## 影響

Borrow/lifetime checking is unsound for enum destructuring: code can move an owner while a reference extracted from an enum payload remains live.

## 修正方針

Carry borrow origins returned by the match scrutinee into payload bind locals, retaining them for the arm scope and releasing them when the arm scope exits.

## 検証

Add compile_fail tests where a reference payload binding remains live while its owner is moved, plus a passing last-use release case.

## 解決

- `visit_expr` の iterative fast path を、式の型が reference を含まない場合だけ使うようにし、reference を含む enum / struct / tuple / reference 変数から borrow origin が失われないようにした。
- `HirExprKind::Match` は scrutinee から返った borrow origin を payload bind local に retain して宣言するようにした。
- payload binding の最後の使用後は既存の last-use release により owner move が可能になることも回帰テストで固定した。

## 修正後検証

- `cargo fmt --all --check`: pass
- `cargo test -p nepl-core --test move_check -- --nocapture`: 47/47 passed
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/move_check.n.md -i tests/compiler/move_effect.n.md --no-tree -o tmp/match-payload-borrow-origin-tests.json -j 1`: 75/75 passed
