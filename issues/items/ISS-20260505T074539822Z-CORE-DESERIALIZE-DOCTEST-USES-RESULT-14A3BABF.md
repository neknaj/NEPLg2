---
id: ISS-20260505T074539822Z-CORE-DESERIALIZE-DOCTEST-USES-RESULT-14A3BABF
title: "core deserialize doctest uses result_exit_code without stdout report"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-05
updated: 2026-05-05
target: stdlib/core/traits/deserialize.nepl
---

# ISS-20260505T074539822Z-CORE-DESERIALIZE-DOCTEST-USES-RESULT-14A3BABF: core deserialize doctest uses result_exit_code without stdout report

## 概要

The deserialize doc-comment doctest returns result_exit_code over a Result<(),str> check, so success is only observed through exit code and no deterministic std/test assertion report is emitted.

## 対象

- `stdlib/core/traits/deserialize.nepl`

## 根拠

- `stdlib/core/traits/deserialize.nepl` の doc-comment doctest は `deserialize<i32> "42"` の結果を `Result<(),str>` に変換し、`result_exit_code check` だけを返していた。
- 失敗 path では `StdErrorKind` を match していたが、stdout report を持たないため成功時も失敗時も fixture 上の assertion report が固定されなかった。

## 問題

The deserialize doc-comment doctest returns result_exit_code over a Result<(),str> check, so success is only observed through exit code and no deterministic std/test assertion report is emitted.

## 影響

Deserialize trait examples can diverge between Rust and selfhost runners in assertion reporting, and the exhaustive StdErrorKind failure branch is not visible in stdout fixtures.

## 修正方針

Convert the doctest to a TestReport via checks_push, preserve exhaustive StdErrorKind matching for failure diagnostics, and assert the checks_print_report stdout fixture with exit_code metadata.

## 対応結果

- doctest metadata に `exit_code: 0` と `Checked [ok]` stdout fixture を追加した。
- `Result<(),str>` + `result_exit_code` をやめ、`checks_push checks_new ...` で `TestReport` を返す形に変更した。
- `StdErrorKind` の failure branch は enum match のまま残し、各 branch は失敗 assertion を report へ積むようにした。

## 検証

- `node nodesrc/tests.js -i stdlib/core/traits/deserialize.nepl --no-tree -o tmp/core-deserialize-report-agent1.json -j 1 --dist web/dist`: total=1, passed=1
- `node nodesrc/test_doctest_std_test_assertion_report_contract.js`: passed
