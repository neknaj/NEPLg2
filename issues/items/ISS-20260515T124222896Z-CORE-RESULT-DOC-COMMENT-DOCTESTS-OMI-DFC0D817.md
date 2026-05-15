---
id: ISS-20260515T124222896Z-CORE-RESULT-DOC-COMMENT-DOCTESTS-OMI-DFC0D817
title: "core result doc-comment doctests omit stdout assertion reports"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-15
updated: 2026-05-15
target: "stdlib/core/result.nepl, nodesrc/test_core_result_doc_report_contract.js"
---

# ISS-20260515T124222896Z-CORE-RESULT-DOC-COMMENT-DOCTESTS-OMI-DFC0D817: core result doc-comment doctests omit stdout assertion reports

## 概要

stdlib/core/result.nepl uses std/test Checks in public doc-comment doctests but keeps ret-only or stdout-less manifests, so Result helper behavior is not pinned as a deterministic assertion report.

## 対象

- `stdlib/core/result.nepl, nodesrc/test_core_result_doc_report_contract.js`

## 根拠

- `stdlib/core/result.nepl` の public doc-comment doctest は `std/test` の `Checks` / assertion helper を使っていた。
- `core_result_basic` と `core_result_and_then` は `ret: 0` のみ、`core_result_map` と `uwok` 例は stdout / exit_code を持たない manifest で、assertion の label / expected / actual を fixture に固定していなかった。
- compile_fail doctest は型エラーと Resource IR move violation の拒否境界なので、stdout report 移行対象ではない。

## 問題

stdlib/core/result.nepl uses std/test Checks in public doc-comment doctests but keeps ret-only or stdout-less manifests, so Result helper behavior is not pinned as a deterministic assertion report.

## 影響

Result is a core abstraction for safe stdlib APIs; self-host runner parity and debugging suffer if ok/err/map/and_then regressions only surface as a 0/1 return value.

## 修正方針

Migrate Result public doc-comment doctests to named TestReport stdout plus exit_code: 0, preserve compile_fail diagnostics, and add a parser-level source policy contract.

## 検証

- `node nodesrc/test_core_result_doc_report_contract.js`
- `node nodesrc/test_doctest_std_test_assertion_report_contract.js`
- `node nodesrc/run_doctest.js -i stdlib/core/result.nepl -n 1 --assert-io --dist web/dist`
- `node nodesrc/run_doctest.js -i stdlib/core/result.nepl -n 3 --assert-io --dist web/dist`
- `node nodesrc/run_doctest.js -i stdlib/core/result.nepl -n 4 --assert-io --dist web/dist`
- `node nodesrc/run_doctest.js -i stdlib/core/result.nepl -n 7 --assert-io --dist web/dist`

## 解決

2026-05-15 に `stdlib/core/result.nepl` の成功系 public doc-comment doctest 4 件を `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` つきの `TestReport` 形式へ移行した。

`ok` / `err` / `unwrap_ok` / `unwrap_err` / `unwrap_or`、`map` / `map_err`、`and_then`、`uwok` alias の観測値を assertion label と expected / actual として stdout に固定した。compile_fail doctest は診断境界を保つため変更していない。`nodesrc/test_core_result_doc_report_contract.js` を追加し、`ret:` や `checks_exit_code` へ戻る退行を検出する。
