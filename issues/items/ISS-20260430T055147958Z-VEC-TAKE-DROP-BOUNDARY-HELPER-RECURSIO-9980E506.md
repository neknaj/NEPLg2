---
id: ISS-20260430T055147958Z-VEC-TAKE-DROP-BOUNDARY-HELPER-RECURSIO-9980E506
title: "Vec take/drop boundary helper recursion forces std/test avoidance"
area: stdlib
status: fixed
resolved: true
priority: P2
type: test
created: 2026-04-30
updated: 2026-04-30
target: stdlib/alloc/collections/vec.nepl
---

# ISS-20260430T055147958Z-VEC-TAKE-DROP-BOUNDARY-HELPER-RECURSIO-9980E506: Vec take/drop boundary helper recursion forces std/test avoidance

## 概要

`stdlib/tests/vec.n.md` の functional helper timeout は remote main で分割済みだが、`take_while` / `drop_while` の block は `std/test` を使わず `if ok 0 1` に戻している。根本には `vec_take_while_len_impl` が再帰で境界探索しており、`std/test` の `check_eq_i32` と併用すると timeout しやすい問題が残っている。

## 対象

- `stdlib/alloc/collections/vec.nepl`
- `stdlib/tests/vec.n.md`

## 根拠

- remote main の `stdlib/tests/vec.n.md` では `vec_prefix_helpers` / `vec_drop_while_helper` が `std/test` を import せず、stdout report を出さない `if ok 0 1` 形式になっている。
- 同じ take/drop 検査を `std/test` の `check_eq_i32` / `checks_print_report` と併用すると、再帰版 `vec_take_while_len_impl` では wasm doctest timeout に到達することを確認した。
- `take_while` / `drop_while` 自体は単独では戻るため、API 仕様ではなく内部 helper の再帰形と test/reporting 組み合わせによる負荷が問題。

## 問題

テストを分割しても、prefix/drop だけ標準の `std/test` report へ載せられない状態が残る。これは `.n.md` test を stdout report と exit code で運用する方針に反し、将来の assert/test 共通化でも同じ回避を温存する。

## 影響

`take_while` / `drop_while` の回帰テストだけが他の Vec functional helper と異なる形式になり、失敗時の詳細確認と test harness の統一運用が弱くなる。selfhost 側で同じ stdlib test を共有する際にも、根本原因が再発しやすい。

## 修正方針

- `vec_take_while_len_impl` を同じ仕様の while loop に変更し、境界探索を iterative にする。
- `vec_prefix_helpers` / `vec_drop_while_helper` を `std/test` の `checks_*` report 形式へ戻し、stdout report と exit code の両方で検証できるようにする。
- Vec module doctest、stdlib Vec test、collection Vec test、source policy を再実行する。

## 修正

- `vec_take_while_len_impl` を再帰から while loop に変更した。
- 境界探索の仕様は、`idx` から開始して最初に predicate が false になる位置、または `len` を返す形で維持した。
- `vec_prefix_helpers` と `vec_drop_while_helper` を `std/test` の `checks_new` / `checks_push` / `checks_print_report` / `checks_exit_code` 形式へ戻した。

## 検証

- `node nodesrc/tests.js -i stdlib/tests/vec.n.md --no-tree -o tmp/vec-take-drop-iterative-stdlib-2.json -j 1 --dist web/dist`: `total=6`, `passed=6`
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec.nepl --no-tree -o tmp/vec-take-drop-iterative-module.json -j 1 --dist web/dist`: `total=37`, `passed=37`
- `node nodesrc/tests.js -i tests/stdlib/vec_collections.n.md --no-tree -o tmp/vec-take-drop-iterative-collections.json -j 1 --dist web/dist`: `total=3`, `passed=3`
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_vec_borrowed_observers.js`: passed
- `node nodesrc/run_source_policy_regressions.js`: passed
- `node nodesrc/issues.js check`: passed (`files=445`)
- `git diff --check`: passed
