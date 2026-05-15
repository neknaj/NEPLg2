---
id: ISS-20260515T125018872Z-STRING-TRIM-DOC-COMMENT-DOCTEST-OMIT-E2099223
title: "string trim doc-comment doctest omits stdout report and public trim coverage"
area: TEST
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-15
updated: 2026-05-15
target: stdlib/alloc/string/slice/trim.nepl
---

# ISS-20260515T125018872Z-STRING-TRIM-DOC-COMMENT-DOCTEST-OMIT-E2099223: string trim doc-comment doctest omits stdout report and public trim coverage

## 概要

The string trim doc-comment test relies on ret: 0 and only exercises str_trim_suffix_cr. It does not pin assertion labels/expected/actual in stdout and does not document-test str_slice_trim_suffix_cr or str_trim.

## 対象

- `stdlib/alloc/string/slice/trim.nepl`

## 根拠

- `stdlib/alloc/string/slice/trim.nepl` の既存 doc-comment doctest は `ret: 0` だけを期待し、assertion label / expected / actual を stdout に固定していなかった。
- 既存 doctest は `str_trim_suffix_cr` の正常系だけを扱い、同じ module の public API である `str_slice_trim_suffix_cr` と `str_trim` の典型的な使い方を documentation test として示していなかった。

## 問題

The string trim doc-comment test relies on ret: 0 and only exercises str_trim_suffix_cr. It does not pin assertion labels/expected/actual in stdout and does not document-test str_slice_trim_suffix_cr or str_trim.

## 影響

Regressions in trim semantics can appear only as an undifferentiated return-value mismatch, and public trim APIs can drift without a documentation-test contract.

## 修正方針

Migrate the trim doc-comment doctest to a named std/test report, pin stdout and exit_code, cover suffix CR trimming, slice+trim bounds, ASCII whitespace trimming, and add a nodesrc contract that rejects ret/checks_exit_code regressions.

## 検証

Run the new nodesrc contract, focused trim doctest, source policy syntax checks, issue index check, and diff whitespace check.

## 対応

`stdlib/alloc/string/slice/trim.nepl` の doc-comment doctest を `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、canonical `TestReport` stdout へ移行した。

- `str_trim_suffix_cr` は CR あり / CR なしの両方を assertion label として固定した。
- `str_slice_trim_suffix_cr` は範囲切り出し後の CR 除去と負の start clamp を固定した。
- `str_trim` は ASCII space/tab/LF/CR の trimming と interior space を保持する挙動を固定した。
- `nodesrc/test_string_trim_doc_report_contract.js` を追加し、`ret:` や `checks_exit_code` へ戻る退行を検出する。
- `nodesrc/run_source_policy_regressions.js` に同 contract を追加した。

## 検証結果

- `node nodesrc/test_string_trim_doc_report_contract.js`: pass
- `node --check nodesrc/test_string_trim_doc_report_contract.js`: pass
- `node --check nodesrc/run_source_policy_regressions.js`: pass
- `node nodesrc/run_doctest.js -i stdlib/alloc/string/slice/trim.nepl -n 1 --assert-io --dist web/dist`: pass
