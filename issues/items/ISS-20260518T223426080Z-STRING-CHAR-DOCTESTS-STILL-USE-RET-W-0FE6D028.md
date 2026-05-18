---
id: ISS-20260518T223426080Z-STRING-CHAR-DOCTESTS-STILL-USE-RET-W-0FE6D028
title: "string char doctests still use ret without stdout report"
area: TEST
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "tests/stdlib/string_char.n.md, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md"
---

# ISS-20260518T223426080Z-STRING-CHAR-DOCTESTS-STILL-USE-RET-W-0FE6D028: string char doctests still use ret without stdout report

## 概要

tests/stdlib/string_char.n.md uses std/test checks but three doctests still specify ret: 0 and do not call checks_print_report, so successful assertion details are not fixed in stdout.

## 対象

- `tests/stdlib/string_char.n.md, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md`

## 根拠

- `tests/stdlib/string_char.n.md` の 3 doctest は `std/test` の `Checks` を構築しているが、manifest は `ret: 0` のみで stdout 期待値を持っていなかった。
- 成功時の assertion report が fixture に固定されないため、Rust runner と selfhost runner が同じ exit code を返しても、string/char API の観測内容や report format の退行を検出できなかった。

## 問題

tests/stdlib/string_char.n.md uses std/test checks but three doctests still specify ret: 0 and do not call checks_print_report, so successful assertion details are not fixed in stdout.

## 影響

Rust runner and selfhost runner can agree on exit status while diverging in assertion report formatting or missing detail for string/char APIs.

## 修正方針

Migrate the three doctests to neplg2:test[stdio, normalize_newlines], exit_code: 0, deterministic stdout, and checks_print_report before checks_exit_code.

## 修正内容

- `tests/stdlib/string_char.n.md` の 3 doctest を `neplg2:test[stdio, normalize_newlines]` + `exit_code: 0` + deterministic `stdout:` に移行した。
- `checks_print_report` の結果を `checks_exit_code` へ渡す形にし、report 出力と exit code 判定を同じ `Checks` owner の流れに統一した。
- `nodesrc/test_stdlib_string_char_report_contract.js` を追加し、対象 doctest が `ret:` へ戻らないこと、stdout report を固定すること、report print 後に exit code を返すことを source policy として検査する。
- `nodesrc/run_source_policy_regressions.js` に同 contract を登録した。

## 検証

- `node nodesrc/test_stdlib_string_char_report_contract.js`
- `node nodesrc/tests.js -i tests/stdlib/string_char.n.md --no-tree -o tmp/agent1-string-char-report.json -j 1 --dist web/dist --assert-io`
