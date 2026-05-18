---
id: ISS-20260518T232455691Z-TRAITS-HASH-DOCTESTS-PRINT-REPORTS-W-234B8FA6
title: "traits_hash doctests print reports without stdout fixtures"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-18
updated: 2026-05-18
target: tests/stdlib/traits_hash.n.md
---

# ISS-20260518T232455691Z-TRAITS-HASH-DOCTESTS-PRINT-REPORTS-W-234B8FA6: traits_hash doctests print reports without stdout fixtures

## 概要

traits_hash.n.md の runtime doctest 3 件は checks_print_report を呼ぶが manifest に stdout と exit_code を固定しておらず、report format や assertion count の退行を検出できない。

## 対象

- `tests/stdlib/traits_hash.n.md`

## 根拠

- `tests/stdlib/traits_hash.n.md` の runtime doctest#1/#5/#6 は `checks_print_report` と `checks_exit_code` を呼んでいた。
- しかし manifest に `stdio` / `normalize_newlines` / `stdout:` / `exit_code:` がなく、runner は report stdout を期待値として比較していなかった。
- `HashKey` / `Hasher` は抽象化機能の安全性を確認する重要 fixture なので、assertion count と report format を固定する必要がある。

## 問題

traits_hash.n.md の runtime doctest 3 件は checks_print_report を呼ぶが manifest に stdout と exit_code を固定しておらず、report format や assertion count の退行を検出できない。

## 影響

HashKey / Hasher abstraction と hash collection の重要な回帰テストが exit status だけに近い扱いになり、stdout report 互換性を selfhost runner と共有できない。

## 修正方針

対象 doctest を stdio + normalize_newlines + exit_code: 0 + deterministic stdout fixture に移行し、source policy regression で ret 代用と stdout 欠落を拒否する。

## 検証

node nodesrc/test_stdlib_traits_hash_report_contract.js; node nodesrc/tests.js -i tests/stdlib/traits_hash.n.md --no-tree -o tmp/agent1-traits-hash-report.json -j 1 --dist web/dist --assert-io

## 2026-05-18 修正

runtime doctest 3 件を `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` fixture に移行した。

- primitive hash trait: 3 assertions
- custom HashMap key/hasher: 3 assertions
- custom HashSet key/hasher: 2 assertions

`nodesrc/test_stdlib_traits_hash_report_contract.js` を追加し、対象 doctest が `ret:` 代用や stdout 欠落へ戻らないことを source policy regression に登録した。
