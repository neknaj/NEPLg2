---
id: ISS-20260518T230932301Z-FS-DOCTESTS-PRINT-REPORTS-BUT-KEEP-R-9E9FDEC4
title: "fs doctests print reports but keep ret metadata without stdout fixtures"
area: TEST
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "tests/stdlib/fs.n.md, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md"
---

# ISS-20260518T230932301Z-FS-DOCTESTS-PRINT-REPORTS-BUT-KEEP-R-9E9FDEC4: fs doctests print reports but keep ret metadata without stdout fixtures

## 概要

tests/stdlib/fs.n.md prints std/test reports in its fs facade doctests, but the manifest still relies on ret metadata or omits stdout/exit_code, so filesystem behavior is not pinned as stdout fixtures.

## 対象

- `tests/stdlib/fs.n.md, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md`

## 根拠

- `tests/stdlib/fs.n.md` の 8 doctest は `checks_print_report` を呼んでいたが、manifest は `ret:` を使うか stdout / exit code expectation を省略していた。
- read / write / normalize / directory sort の成功時 report が fixture に固定されないため、runner 間の stdout compatibility を検出できなかった。

## 問題

tests/stdlib/fs.n.md prints std/test reports in its fs facade doctests, but the manifest still relies on ret metadata or omits stdout/exit_code, so filesystem behavior is not pinned as stdout fixtures.

## 影響

Rust and selfhost runners can agree on exit status while losing detailed fs assertion reports for reads, writes, normalization, and sorting.

## 修正方針

Migrate fs facade doctests to stdio + exit_code + deterministic stdout and add a source policy contract.

## 修正内容

- `tests/stdlib/fs.n.md` の 8 doctest を `neplg2:test[stdio, normalize_newlines]` + `exit_code: 0` + deterministic `stdout:` へ移行した。
- skip 中の `fs_read_dir_returns_sorted_entries` も manifest contract として stdout expectation を持たせ、skip tag と stdio tag の両方を保持した。
- `nodesrc/test_stdlib_fs_report_contract.js` を追加し、`ret:` 再導入、stdout fixture 欠落、report 出力なしの exit-code-only 退行を source policy で拒否する。
- `nodesrc/run_source_policy_regressions.js` に同 contract を登録した。

## 検証

- `node nodesrc/test_stdlib_fs_report_contract.js`
- `node nodesrc/tests.js -i tests/stdlib/fs.n.md --no-tree -o tmp/agent1-fs-nmd-report.json -j 1 --dist web/dist --assert-io`
