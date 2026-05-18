---
id: ISS-20260518T230457387Z-TRAITS-ORDER-VEC-SORT-DOCTEST-HIDES--411C5DF7
title: "traits_order vec sort doctest hides assertion detail behind ret metadata"
area: TEST
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "tests/stdlib/traits_order.n.md, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md"
---

# ISS-20260518T230457387Z-TRAITS-ORDER-VEC-SORT-DOCTEST-HIDES--411C5DF7: traits_order vec sort doctest hides assertion detail behind ret metadata

## 概要

tests/stdlib/traits_order.n.md::vec sort imports std/test but reduces four sorted-order observations to a single bool and ret: 0, so stdout does not show which ordered position regressed.

## 対象

- `tests/stdlib/traits_order.n.md, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md`

## 根拠

- `tests/stdlib/traits_order.n.md` の vec sort doctest は 4 つの位置確認を `b0..b3` に畳み込み、最後に 1 つの bool と `ret: 0` だけで成功を表していた。
- `std/test` を import しているにもかかわらず stdout report を出していないため、どの順序位置が壊れたかが fixture 上で確認できなかった。

## 問題

tests/stdlib/traits_order.n.md::vec sort imports std/test but reduces four sorted-order observations to a single bool and ret: 0, so stdout does not show which ordered position regressed.

## 影響

Trait/Ord sorting regressions can agree on exit status while hiding assertion detail from Rust and selfhost runners.

## 修正方針

Use std/test Checks for each sorted position, print the report, migrate to stdio + exit_code + deterministic stdout, and add a source policy contract.

## 修正内容

- vec sort doctest を `neplg2:test[stdio, normalize_newlines]` + `exit_code: 0` + deterministic `stdout:` へ移行した。
- 4 つの sorted-position check を `std/test::Checks` へ分解し、`checks_print_report` の結果を `checks_exit_code` へ渡すようにした。
- `nodesrc/test_stdlib_traits_order_report_contract.js` を追加し、`ret:` 再導入、stdout fixture 欠落、report 出力なしの exit-code-only 退行を source policy で拒否する。
- `nodesrc/run_source_policy_regressions.js` に同 contract を登録した。

## 検証

- `node nodesrc/test_stdlib_traits_order_report_contract.js`
- `node nodesrc/tests.js -i tests/stdlib/traits_order.n.md --no-tree -o tmp/agent1-traits-order-report.json -j 1 --dist web/dist --assert-io`
