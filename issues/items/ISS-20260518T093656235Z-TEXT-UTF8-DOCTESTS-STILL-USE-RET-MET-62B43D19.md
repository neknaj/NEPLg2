---
id: ISS-20260518T093656235Z-TEXT-UTF8-DOCTESTS-STILL-USE-RET-MET-62B43D19
title: "text_utf8 doctests still use ret metadata instead of stdout reports"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-18
updated: 2026-05-18
target: "tests/stdlib/text_utf8.n.md, nodesrc/test_stdlib_text_utf8_report_contract.js"
---

# ISS-20260518T093656235Z-TEXT-UTF8-DOCTESTS-STILL-USE-RET-MET-62B43D19: text_utf8 doctests still use ret metadata instead of stdout reports

## 概要

tests/stdlib/text_utf8.n.md still uses ret: 0 as the assertion-suite success contract, and several cases compute Checks without printing the report. This leaves the UTF-8 boundary regression suite dependent on exit status only instead of pinning the assertion report in stdout.

## 対象

- `tests/stdlib/text_utf8.n.md, nodesrc/test_stdlib_text_utf8_report_contract.js`

## 根拠

- `tests/stdlib/text_utf8.n.md` の 9 doctest はすべて `ret: 0` を使っていた。
- `text_utf8_decode_next_reads_char_offsets` と `text_utf8_encode_char_returns_bytebuf` は `Checks` を作っていたが `checks_print_report` を呼ばず、成功時 stdout に assertion detail が出なかった。
- この file は `str` の UTF-8 invariant、invalid UTF-8 rejection、file/std/io text conversion boundary を監視する safety regression なので、exit status だけでは検査内容の退行を検出しにくい。

## 問題

tests/stdlib/text_utf8.n.md still uses ret: 0 as the assertion-suite success contract, and several cases compute Checks without printing the report. This leaves the UTF-8 boundary regression suite dependent on exit status only instead of pinning the assertion report in stdout.

## 影響

The UTF-8 safety suite may pass with the same exit code while report formatting, assertion count, or report emission regresses. This weakens the shared Rust/selfhost .n.md contract for source text and invalid UTF-8 diagnostics.

## 修正方針

Migrate all text_utf8 doctests to neplg2:test[stdio, normalize_newlines], stdout fixture, and exit_code: 0. Ensure each case prints checks_print_report before checks_exit_code. Add a source policy that rejects ret: metadata and missing stdout reports for this file.

## 検証

Run the new source policy and focused text_utf8 doctests with --assert-io.

## 修正内容

- 9 doctest すべてを `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` fixture へ移行した。
- report を出していなかった 2 case は `checks_print_report` の結果を `checks_exit_code` へ渡す形にし、UTF-8 decode / encode の assertion count を stdout に固定した。
- `nodesrc/test_stdlib_text_utf8_report_contract.js` を追加し、`ret:` 再導入、stdout fixture 欠落、`checks_print_report` なしの exit-code-only 退行を検出するようにした。
- `nodesrc/run_source_policy_regressions.js` に同 policy を登録した。

## 検証結果

- `node nodesrc/test_stdlib_text_utf8_report_contract.js`: passed
- `node nodesrc/tests.js -i tests/stdlib/text_utf8.n.md --no-tree -o tmp/agent1-text-utf8-report-metadata.json -j 1 --dist web/dist --assert-io`: total=9, passed=9
