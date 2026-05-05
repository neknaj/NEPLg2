---
id: ISS-20260505T072734472Z-CORE-RESULT-DOCTESTS-OMIT-STDOUT-ASS-1D20CA46
title: "core result doctests omit stdout assertion reports"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-05
updated: 2026-05-05
target: stdlib/core/result.nepl
---

# ISS-20260505T072734472Z-CORE-RESULT-DOCTESTS-OMIT-STDOUT-ASS-1D20CA46: core result doctests omit stdout assertion reports

## 概要

The std-target Result doc-comment doctests use std/test checks but either rely on ret: 0 or have no explicit exit_code/stdout metadata, and return checks_exit_code checks without printing assertion reports.

## 対象

- `stdlib/core/result.nepl`

## 根拠

- `stdlib/core/result.nepl` には runnable な std-target `std/test` doctest が 4 件ある。
- 修正前は 2 件が `ret: 0` を使い、2 件は explicit metadata なしで、いずれも `checks_exit_code checks` だけを返していた。
- 同ファイルには compile_fail と `#target core` の戻り値確認も含まれるため、stdout report 移行対象を runnable std/test doctest に限定する必要があった。

## 問題

The std-target Result doc-comment doctests use std/test checks but either rely on ret: 0 or have no explicit exit_code/stdout metadata, and return checks_exit_code checks without printing assertion reports.

## 影響

Result helper regressions can pass or fail only through exit success while assertion report output remains unverified, and documentation still refers to ret-based success for std doctests.

## 修正方針

Migrate only runnable std/test Result doctests to exit_code metadata plus deterministic checks_print_report stdout fixtures, leaving compile_fail and core return-value tests in their own roles.

## 対応結果

- runnable std-target `std/test` doctest 4 件を `exit_code: 0` + `stdout: mlstr:` に移行した。
- 5 件 / 2 件 / 2 件 / 1 件の assertion report を fixture として固定した。
- compile_fail と `#target core` の戻り値確認は責務が異なるため変更しなかった。
- 注意書きを `ret:` 比較ではなく stdout report と `exit_code:` で確認する説明へ更新した。

## 検証

- `node nodesrc/tests.js -i stdlib/core/result.nepl --no-tree -o tmp/core-result-report-agent1.json -j 1 --dist web/dist`: total=7, passed=7
- `node nodesrc/test_doctest_std_test_assertion_report_contract.js`: passed
