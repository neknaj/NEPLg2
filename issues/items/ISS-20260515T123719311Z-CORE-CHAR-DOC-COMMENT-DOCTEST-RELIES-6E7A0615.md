---
id: ISS-20260515T123719311Z-CORE-CHAR-DOC-COMMENT-DOCTEST-RELIES-6E7A0615
title: "core char doc-comment doctest relies on ret-only assertion report"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-15
updated: 2026-05-15
target: "stdlib/core/char.nepl, nodesrc/test_core_char_doc_report_contract.js"
---

# ISS-20260515T123719311Z-CORE-CHAR-DOC-COMMENT-DOCTEST-RELIES-6E7A0615: core char doc-comment doctest relies on ret-only assertion report

## 概要

stdlib/core/char.nepl imports std/test and builds a Checks suite, but the doc-comment doctest only asserts ret: 0 and does not pin the stdout assertion report.

## 対象

- `stdlib/core/char.nepl, nodesrc/test_core_char_doc_report_contract.js`

## 根拠

- `stdlib/core/char.nepl` の file-level doctest は `std/test` を import し、`Checks` に `assert_*` を集約していた。
- しかし manifest は `ret: 0` のみで、stdout に各 assertion の label / expected / actual を固定していなかった。
- `core/option` などの移行済み doc-comment doctest は `test_report_print_stdout` + `test_report_exit_code` の canonical 形式になっている。

## 問題

stdlib/core/char.nepl imports std/test and builds a Checks suite, but the doc-comment doctest only asserts ret: 0 and does not pin the stdout assertion report.

## 影響

A regression in char classification or report formatting could still return exit code 0 without fixture-level expected/actual output, weakening self-host runner parity.

## 修正方針

Migrate the char public doc-comment doctest to canonical std/test stdout report plus exit_code: 0, and add a source policy contract preventing ret-only regression.

## 検証

- `node nodesrc/test_core_char_doc_report_contract.js`
- `node nodesrc/test_doctest_std_test_assertion_report_contract.js`
- `node nodesrc/run_doctest.js -i stdlib/core/char.nepl -n 1 --assert-io --dist web/dist`

## 解決

2026-05-15 に `stdlib/core/char.nepl` の public doc-comment doctest を `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` つきの `TestReport` 形式へ移行した。

`char_to_i32`、ASCII 分類、UTF-8 byte length、valid / invalid scalar conversion の9観測値を assertion label と expected / actual として stdout に固定した。`nodesrc/test_core_char_doc_report_contract.js` を追加し、同 doctest が `ret:` や `checks_exit_code` へ戻る退行を検出する。
