---
id: ISS-20260519T053837519Z-UTF-8-VALIDATION-DOC-COMMENT-DOCTEST-5BD9215D
title: "UTF-8 validation doc-comment doctests still use ret-only metadata"
area: TEST
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-19
updated: 2026-05-19
target: "stdlib/std/text/validate.nepl, stdlib/alloc/string/utf8.nepl, nodesrc/test_stdlib_utf8_validation_doc_report_contract.js, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md"
---

# ISS-20260519T053837519Z-UTF-8-VALIDATION-DOC-COMMENT-DOCTEST-5BD9215D: UTF-8 validation doc-comment doctests still use ret-only metadata

## 概要

UTF-8 validation doc-comment doctests still encoded success as ret-only or no explicit stdout contract, so report details were invisible to runner and selfhost migration.

## 対象

- `stdlib/std/text/validate.nepl, stdlib/alloc/string/utf8.nepl, nodesrc/test_stdlib_utf8_validation_doc_report_contract.js, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md`

## 根拠

- `stdlib/std/text/validate.nepl` は module-level doctest が stdout / exit_code expectation を持たず、3 つの API doc-comment doctest は `ret: 1` だけで成功を表していた。
- `stdlib/alloc/string/utf8.nepl` は leading byte 分類と memory validation の doc-comment doctest が `ret: 1` だけで成功を表していた。
- `nodesrc/test_nepl_doc_report_metadata_policy.js` は report helper を使う doctest の metadata は検出できるが、report helper 自体を使わない ret-only doctest は代表 fixture ごとの contract で段階移行する必要がある。

## 問題

UTF-8 validation doc-comment doctests encoded success as ret-only or no explicit stdout contract, so report details were invisible to runner and selfhost migration.

## 影響

A regression in UTF-8 lead classification or validation can be hidden behind a numeric return value, and report metadata policy does not pin these representative stdlib doc-comment fixtures.

## 修正方針

Migrate the doctests to named TestReport stdout fixtures with exit_code: 0 and add a source policy contract for the UTF-8 validation docs.

## 検証

Run the new source policy contract and focused doctests for stdlib/std/text/validate.nepl and stdlib/alloc/string/utf8.nepl.

## 修正内容

- `stdlib/std/text/validate.nepl` の 4 doctest を `neplg2:test[stdio, normalize_newlines]` + `exit_code: 0` + deterministic `stdout:` へ移行した。
- `stdlib/alloc/string/utf8.nepl` の 2 doctest を同じ canonical stdout report 形式へ移行した。
- 各 doctest は `std/test::TestReport` で、UTF-8 bytes / leading byte classification / byte-at / memory span validation の観測結果を stdout に固定する。
- `nodesrc/test_stdlib_utf8_validation_doc_report_contract.js` を追加し、対象 doctest の `ret:` 再導入、stdout fixture 欠落、旧 `checks_*` 形式への退行を拒否する。
- `nodesrc/run_source_policy_regressions.js` に同 contract を登録した。

## 検証結果

- `node nodesrc/test_stdlib_utf8_validation_doc_report_contract.js`: passed
- `node nodesrc/tests.js -i stdlib\std\text\validate.nepl -i stdlib\alloc\string\utf8.nepl --no-tree -o tmp\agent1-utf8-validation-doc-report.json -j 1 --dist web\dist --assert-io`: total=6, passed=6
