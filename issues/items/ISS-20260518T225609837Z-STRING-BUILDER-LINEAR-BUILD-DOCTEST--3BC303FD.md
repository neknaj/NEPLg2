---
id: ISS-20260518T225609837Z-STRING-BUILDER-LINEAR-BUILD-DOCTEST--3BC303FD
title: "string builder linear build doctest hides std/test report behind ret metadata"
area: TEST
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "tests/stdlib/string.n.md, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md"
---

# ISS-20260518T225609837Z-STRING-BUILDER-LINEAR-BUILD-DOCTEST--3BC303FD: string builder linear build doctest hides std/test report behind ret metadata

## 概要

tests/stdlib/string.n.md::test_string_builder_linear_build builds a std/test Checks report but returns checks_exit_code directly under ret: 0, so the long StringBuilder result assertion is not pinned in stdout.

## 対象

- `tests/stdlib/string.n.md, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md`

## 根拠

- `tests/stdlib/string.n.md::test_string_builder_linear_build` は `StringBuilder` で 2000 文字を構築して `len out == 2000` を検査しているが、`ret: 0` と `checks_exit_code checks` だけで成功を表していた。
- stdout expectation がないため、長い builder path の assertion report と runner 間の report compatibility が fixture に残らなかった。

## 問題

tests/stdlib/string.n.md::test_string_builder_linear_build builds a std/test Checks report but returns checks_exit_code directly under ret: 0, so the long StringBuilder result assertion is not pinned in stdout.

## 影響

StringBuilder linear build regressions can match exit status while losing assertion report compatibility for Rust and selfhost runners.

## 修正方針

Print the Checks report, migrate the doctest to stdio + exit_code + deterministic stdout, and add a source policy contract.

## 修正内容

- `test_string_builder_linear_build` を `neplg2:test[stdio, normalize_newlines]` + `exit_code: 0` + deterministic `stdout:` へ移行した。
- `checks_print_report` の結果を `checks_exit_code` へ渡す形にし、`StringBuilder` 長大 append の検査結果を stdout fixture に固定した。
- `nodesrc/test_stdlib_string_nmd_report_contract.js` を追加し、対象 doctest が `ret:` へ戻らないこと、stdout report を固定すること、report print 後に exit code を返すことを source policy として検査する。
- `nodesrc/run_source_policy_regressions.js` に同 contract を登録した。

## 検証

- `node nodesrc/test_stdlib_string_nmd_report_contract.js`
- `node nodesrc/tests.js -i tests/stdlib/string.n.md --no-tree -o tmp/agent1-string-nmd-report.json -j 1 --dist web/dist --assert-io`
