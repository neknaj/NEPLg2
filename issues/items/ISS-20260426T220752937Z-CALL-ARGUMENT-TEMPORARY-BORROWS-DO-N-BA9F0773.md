---
id: ISS-20260426T220752937Z-CALL-ARGUMENT-TEMPORARY-BORROWS-DO-N-BA9F0773
title: "Call argument temporary borrows do not overlap"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-27
target: "nepl-core/src/passes/move_check.rs, nepl-core/tests/move_check.rs, tests/compiler/move_check.n.md"
---

# ISS-20260426T220752937Z-CALL-ARGUMENT-TEMPORARY-BORROWS-DO-N-BA9F0773: Call argument temporary borrows do not overlap

## 概要

move_check checks reference arguments one at a time and does not retain their temporary borrows for the duration of the call, so f &mut x &x and f &x &mut x compile successfully.

## 対象

- `nepl-core/src/passes/move_check.rs, nepl-core/tests/move_check.rs, tests/compiler/move_check.n.md`

## 根拠

- `visit_call_args_with_params` は各引数の `ExprBorrow` を `result_borrows` へ集めるだけで、引数評価中や呼び出し完了まで `borrow_counts` に保持しない。
- `visit_reference_call_arg` は `&mut x` のような `AddrOf` で `visit_temporary_borrow` を呼ぶが、`Valid` な source に対する検査だけで終わり、次の引数を検査する時点で `x` は `BorrowedUnique` になっていない。
- 最小再現として `fn use_both <(&mut LocalToken,&LocalToken)->i32> ...; use_both &mut x &x` が `cargo run -q -p nepl-cli -- --check --target core` で `Check successful` になる。
- 逆順の `fn use_both <(&LocalToken,&mut LocalToken)->i32> ...; use_both &x &mut x` も同じく `Check successful` になる。

## 問題

move_check checks reference arguments one at a time and does not retain their temporary borrows for the duration of the call, so f &mut x &x and f &x &mut x compile successfully.

## 影響

Borrow/lifetime checking is unsound for function calls: shared and unique borrows that should overlap during a call can alias the same owner, and later arguments can move or borrow through an active earlier reference argument.

## 修正方針

Retain borrow origins from call arguments until all arguments have been checked, then release the call-duration temporaries while still returning origins for reference results.

## 検証

Add compile_fail tests for overlapping &mut/& call arguments and a passing test where a temporary reference argument releases after the call.

## 解決

- `visit_call_args_with_params` で、各引数の borrow origin を call-duration の一時 borrow として retain し、全引数の検査が終わってから release するようにした。
- 関数が参照を返す場合の origin は従来どおり `result_borrows` として外側へ返し、call-duration temporary とは分離した。
- `f &mut x &x` と `f &x &mut x` を compile_fail として追加し、既存の「単一 temporary borrow は call 後に解放される」テストも維持した。

## 修正後検証

- `cargo fmt --all --check`: pass
- `cargo test -p nepl-core --test move_check -- --nocapture`: 43/43 passed
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/move_check.n.md -i tests/compiler/move_effect.n.md --no-tree -o tmp/call-argument-temporary-borrow-tests.json -j 1`: 71/71 passed
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass
