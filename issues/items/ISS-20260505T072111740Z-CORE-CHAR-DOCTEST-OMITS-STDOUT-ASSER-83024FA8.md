---
id: ISS-20260505T072111740Z-CORE-CHAR-DOCTEST-OMITS-STDOUT-ASSER-83024FA8
title: "core char doctest omits stdout assertion report"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-05
updated: 2026-05-05
target: stdlib/core/char.nepl
---

# ISS-20260505T072111740Z-CORE-CHAR-DOCTEST-OMITS-STDOUT-ASSER-83024FA8: core char doctest omits stdout assertion report

## 概要

The core char doc-comment doctest uses std/test checks for Unicode scalar and ASCII helper assertions but still records ret: 0 and returns checks_exit_code checks without printing the assertion report.

## 対象

- `stdlib/core/char.nepl`

## 根拠

- `stdlib/core/char.nepl` の doc-comment doctest は `std/test` checks で 9 件の Unicode scalar / ASCII helper assertion を集約していた。
- 修正前は `ret: 0` metadata と `checks_exit_code checks` だけで成功を表し、stdout report を fixture として固定していなかった。

## 問題

The core char doc-comment doctest uses std/test checks for Unicode scalar and ASCII helper assertions but still records ret: 0 and returns checks_exit_code checks without printing the assertion report.

## 影響

Char helper regressions can only be observed through exit success/failure, and Rust/selfhost runner parity does not verify the assertion report output for this core std doctest.

## 修正方針

Migrate the doctest to exit_code: 0, print the deterministic std/test checks report, and assert the stdout fixture.

## 対応結果

- doctest metadata を `ret: 0` から `exit_code: 0` に移行した。
- `checks_print_report` の stdout を `Checked [ok,ok,ok,ok,ok,ok,ok,ok,ok]` fixture として固定した。
- `checks_exit_code` には表示済み report を渡し、stdout report と exit code を同じ check aggregation から導出する形にした。

## 検証

- `node nodesrc/tests.js -i stdlib/core/char.nepl --no-tree -o tmp/core-char-report-agent1.json -j 1 --dist web/dist`: total=1, passed=1
- `node nodesrc/test_doctest_std_test_assertion_report_contract.js`: passed
