---
id: ISS-20260518T224250390Z-NEPLG2-TEXT-DOCTESTS-STILL-USE-RET-M-82850ECD
title: "neplg2_text doctests still use ret metadata instead of stdout reports"
area: TEST
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "tests/stdlib/neplg2_text.n.md, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md"
---

# ISS-20260518T224250390Z-NEPLG2-TEXT-DOCTESTS-STILL-USE-RET-M-82850ECD: neplg2_text doctests still use ret metadata instead of stdout reports

## 概要

tests/stdlib/neplg2_text.n.md already prints std/test Checks reports, but four doctests still use ret: 0 and do not pin stdout, so source-text behavior can diverge between runners without fixture diff.

## 対象

- `tests/stdlib/neplg2_text.n.md, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md`

## 根拠

- `tests/stdlib/neplg2_text.n.md` の 4 doctest は `checks_print_report` を呼んでいるが、manifest が `ret: 0` のままだった。
- stdout expectation がないため、line map / CRLF span / out-of-range / large line-map の assertion detail と report format が runner 互換性として固定されていなかった。

## 問題

tests/stdlib/neplg2_text.n.md already prints std/test Checks reports, but four doctests still use ret: 0 and do not pin stdout, so source-text behavior can diverge between runners without fixture diff.

## 影響

Selfhost source text line-map tests can agree on exit status while losing assertion detail or report formatting compatibility.

## 修正方針

Migrate the four doctests to neplg2:test[stdio, normalize_newlines], exit_code: 0, deterministic stdout, and add a source policy contract.

## 修正内容

- `tests/stdlib/neplg2_text.n.md` の 4 doctest を `neplg2:test[stdio, normalize_newlines]` + `exit_code: 0` + deterministic `stdout:` に移行した。
- 既存の `checks_print_report` -> `checks_exit_code` の実行順は維持し、成功時の assertion 件数と各 `ok` 行を fixture に固定した。
- `nodesrc/test_selfhost_source_text_report_contract.js` を追加し、`ret:` 再導入、stdout fixture 欠落、report 出力なしの exit-code-only 退行を source policy で拒否する。
- `nodesrc/run_source_policy_regressions.js` に同 contract を登録した。

## 検証

- `node nodesrc/test_selfhost_source_text_report_contract.js`
- `node nodesrc/tests.js -i tests/stdlib/neplg2_text.n.md --no-tree -o tmp/agent1-neplg2-text-report.json -j 1 --dist web/dist --assert-io`
