---
id: ISS-20260505T071723578Z-CORE-TEXT-TRAIT-DOCTESTS-OMIT-STDOUT-8D134355
title: "core text trait doctests omit stdout assertion reports"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-05
updated: 2026-05-05
target: "stdlib/core/traits/stringify.nepl, stdlib/core/traits/debug.nepl, stdlib/core/traits/serialize.nepl"
---

# ISS-20260505T071723578Z-CORE-TEXT-TRAIT-DOCTESTS-OMIT-STDOUT-8D134355: core text trait doctests omit stdout assertion reports

## 概要

The Stringify, Debug, and Serialize core trait doc-comment doctests build std/test checks lists but return checks_exit_code checks directly, so assertion success is only observed through exit code and the report format is not fixed.

## 対象

- `stdlib/core/traits/stringify.nepl, stdlib/core/traits/debug.nepl, stdlib/core/traits/serialize.nepl`

## 根拠

- `stdlib/core/traits/stringify.nepl` / `debug.nepl` / `serialize.nepl` の doc-comment doctest は、いずれも `std/test` の `checks_new` / `checks_push` で 2 件の assertion を集約していた。
- 修正前は `checks_exit_code checks` を直接返しており、成功時の `Checked [ok,ok]` report を stdout fixture として固定していなかった。

## 問題

The Stringify, Debug, and Serialize core trait doc-comment doctests build std/test checks lists but return checks_exit_code checks directly, so assertion success is only observed through exit code and the report format is not fixed.

## 影響

Text trait examples can regress their assertion report output without fixture failures, which weakens Rust/selfhost runner parity for core trait documentation tests.

## 修正方針

Emit deterministic checks_print_report output for each doctest, assert the stdout fixture, and keep exit_code as the process-success contract.

## 対応結果

- 3 件の doctest metadata を `exit_code: 0` + `stdout: mlstr:` に変更した。
- 各 doctest で `checks_print_report` の戻り値を `checks_exit_code` に渡し、stdout report と exit code を同じ check aggregation から導出する形へ統一した。

## 検証

- `node nodesrc/tests.js -i stdlib/core/traits/stringify.nepl -i stdlib/core/traits/debug.nepl -i stdlib/core/traits/serialize.nepl --no-tree -o tmp/core-text-traits-report-agent1.json -j 1 --dist web/dist`: total=3, passed=3
- `node nodesrc/test_doctest_std_test_assertion_report_contract.js`: passed
