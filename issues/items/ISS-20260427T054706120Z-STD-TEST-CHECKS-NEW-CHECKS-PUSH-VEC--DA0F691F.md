---
id: ISS-20260427T054706120Z-STD-TEST-CHECKS-NEW-CHECKS-PUSH-VEC--DA0F691F
title: "std/test の checks_new/checks_push が Vec allocation failure を unwrap_ok で trap する"
area: stdlib
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/std/test.nepl, tests/stdlib/std_test_collect.n.md, nodesrc/test_stdlib_std_test_no_unsafe_unwraps.js"
---

# ISS-20260427T054706120Z-STD-TEST-CHECKS-NEW-CHECKS-PUSH-VEC--DA0F691F: std/test の checks_new/checks_push が Vec allocation failure を unwrap_ok で trap する

## 概要

std/test checks_new and checks_push use unwrap_ok around Vec<Result<(),str>> new/push, so the collectable test harness can trap on allocation failure instead of returning a test failure value.

## 対象

- `stdlib/std/test.nepl, tests/stdlib/std_test_collect.n.md, nodesrc/test_stdlib_std_test_no_unsafe_unwraps.js`

## 根拠

- `checks_new` は `unwrap_ok<Vec<Result<(),str>>, StdErrorKind> new<Result<(),str>>` で `Vec` 生成失敗を trap していた。
- `checks_push` は `unwrap_ok push<Result<(),str>> checks r` で grow failure を trap していた。
- `std/test` は多くの doctest から `as *` で import されるため、修正途中の unqualified `new` / `push` は caller 側 collection import と衝突し得ることも確認した。

## 問題

std/test checks_new and checks_push use unwrap_ok around Vec<Result<(),str>> new/push, so the collectable test harness can trap on allocation failure instead of returning a test failure value.

## 影響

stdlib and self-host regression tests depend heavily on checks_new/checks_push. Under memory pressure the test framework can hide the actual failing check behind an unsafe helper trap.

## 修正方針

Replace implementation unwrap_ok with explicit Result matches. Keep the existing Vec-returning API by returning an empty Vec sentinel from checks_new and by appending a Result::Err allocation failure marker when checks_push cannot grow.

## 検証

- `node nodesrc/test_stdlib_std_test_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i stdlib/std/test.nepl --no-tree -o tmp/std-test-allocation-docs-fixed.json -j 1`: 12/12 passed
- `node nodesrc/tests.js -i stdlib/tests/btreeset.n.md --no-tree -o tmp/std-test-btreeset-repro-fixed.json -j 1`: 4/4 passed
- `node nodesrc/tests.js -i tests/stdlib/std_test_collect.n.md --no-tree -o tmp/std-test-allocation-focused-fixed.json -j 1`: 3/3 passed
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-std-test-allocation-fixed.json -j 4`: 305/305 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-std-test-allocation-fixed.json -j 4`: 418/418 passed

## 解決内容

- `checks_empty_vec` を追加し、allocation failure 時に consumed owner を再利用しない空 `Vec<Result<(),str>>` sentinel を明示した。
- `checks_single_error` を追加し、`checks_push` の grow failure を可能な限り `Result::Err "std/test checks_push allocation failed"` として集約結果に戻すようにした。
- `checks_new` / `checks_push` の implementation `unwrap_ok` を `match` に置き換えた。
- `std/test` 内部の Vec 操作を `v::new` / `v::push` / `v::Vec` に限定し、caller 側の `as *` import と `new` / `push` が衝突しないようにした。
- `nodesrc/test_stdlib_std_test_no_unsafe_unwraps.js` を追加し、CI/source policy と `doc/testing.md` に登録した。
