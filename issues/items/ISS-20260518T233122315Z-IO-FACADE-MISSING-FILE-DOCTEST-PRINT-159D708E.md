---
id: ISS-20260518T233122315Z-IO-FACADE-MISSING-FILE-DOCTEST-PRINT-159D708E
title: "io facade missing-file doctest prints report without stdout fixture"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-18
updated: 2026-05-18
target: tests/stdlib/io.n.md
---

# ISS-20260518T233122315Z-IO-FACADE-MISSING-FILE-DOCTEST-PRINT-159D708E: io facade missing-file doctest prints report without stdout fixture

## 概要

tests/stdlib/io.n.md の missing-file doctest は checks_print_report を呼ぶが manifest に stdout と exit_code を固定しておらず、IoError assertion report の退行を検出できない。

## 対象

- `tests/stdlib/io.n.md`

## 根拠

- `tests/stdlib/io.n.md::io_fs_missing_file_is_io_error` は `std/test::Checks` で `IoError` を検査し、`checks_print_report` を呼んでいた。
- しかし manifest に `stdio` / `normalize_newlines` / `stdout:` / `exit_code:` がなく、runner は `Checked [ok]` report を期待値として比較していなかった。
- missing-file error kind は `std/io` facade の重要な失敗系 contract なので、成功時も expected/actual の assertion report を fixture に残す必要がある。

## 問題

tests/stdlib/io.n.md の missing-file doctest は checks_print_report を呼ぶが manifest に stdout と exit_code を固定しておらず、IoError assertion report の退行を検出できない。

## 影響

std/io facade の read error contract が exit status だけに近い扱いになり、失敗時に error kind の expected/actual が fixture 差分として残らない。

## 修正方針

missing-file doctest を stdio + normalize_newlines + exit_code: 0 + deterministic stdout fixture に移行し、source policy regression で stdout 欠落を拒否する。

## 検証

node nodesrc/test_stdlib_io_nmd_report_contract.js; node nodesrc/tests.js -i tests/stdlib/io.n.md --no-tree -o tmp/agent1-io-nmd-report.json -j 1 --dist web/dist --assert-io

## 2026-05-18 修正

`io_fs_missing_file_is_io_error` を `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` fixture に移行した。

`nodesrc/test_stdlib_io_nmd_report_contract.js` を追加し、対象 doctest が `ret:` 代用や stdout 欠落へ戻らないことを source policy regression に登録した。
