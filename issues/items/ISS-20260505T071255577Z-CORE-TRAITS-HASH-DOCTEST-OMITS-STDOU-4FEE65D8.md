---
id: ISS-20260505T071255577Z-CORE-TRAITS-HASH-DOCTEST-OMITS-STDOU-4FEE65D8
title: "core traits hash doctest omits stdout assertion report"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-05
updated: 2026-05-05
target: stdlib/core/traits/hash.nepl
---

# ISS-20260505T071255577Z-CORE-TRAITS-HASH-DOCTEST-OMITS-STDOU-4FEE65D8: core traits hash doctest omits stdout assertion report

## 概要

The hash trait doc-comment doctest builds a std/test checks list but returns checks_exit_code checks directly, so success is only observed through the process exit code and the assertion report format is not fixed in the fixture.

## 対象

- `stdlib/core/traits/hash.nepl`

## 根拠

- `stdlib/core/traits/hash.nepl` の doc-comment doctest は `std/test` の `checks_new` / `checks_push` を使って 2 件の assertion を集約していた。
- 修正前は `checks_exit_code checks` を返すだけで、成功時の `Checked [ok,ok]` report を stdout fixture として固定していなかった。

## 問題

The hash trait doc-comment doctest builds a std/test checks list but returns checks_exit_code checks directly, so success is only observed through the process exit code and the assertion report format is not fixed in the fixture.

## 影響

Rust and selfhost runners can agree on exit success while silently diverging in assertion report output, and regression triage lacks the checked assertion list for this core trait example.

## 修正方針

Emit the deterministic std/test assertion report with checks_print_report, assert the stdout fixture, and keep exit_code as the process-success contract.

## 対応結果

- doctest metadata を `exit_code: 0` にし、process success の期待値を `ret:` ではなく exit code として表した。
- `checks_print_report` の戻り値を `checks_exit_code` に渡す形へ変更し、stdout に `Checked [ok,ok]` と各 assertion の `ok` 行を出す契約を fixture 化した。

## 検証

- `node nodesrc/run_doctest.js -i stdlib/core/traits/hash.nepl -n 1 --dist web/dist`: passed, stdout=`Checked [ok,ok]`
- `node nodesrc/tests.js -i stdlib/core/traits/hash.nepl --no-tree -o tmp/core-traits-hash-report-agent1.json -j 1 --dist web/dist`: total=1, passed=1
- `node nodesrc/test_doctest_std_test_assertion_report_contract.js`: passed
