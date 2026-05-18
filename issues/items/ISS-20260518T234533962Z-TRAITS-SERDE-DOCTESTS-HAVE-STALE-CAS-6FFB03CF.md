---
id: ISS-20260518T234533962Z-TRAITS-SERDE-DOCTESTS-HAVE-STALE-CAS-6FFB03CF
title: "traits_serde doctests have stale cast import and unpinned reports"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-18
updated: 2026-05-18
target: tests/stdlib/traits_serde.n.md
---

# ISS-20260518T234533962Z-TRAITS-SERDE-DOCTESTS-HAVE-STALE-CAS-6FFB03CF: traits_serde doctests have stale cast import and unpinned reports

## 概要

tests/stdlib/traits_serde.n.md の serialize doctest は core/cast を import せず cast を使って compile failure になっている。さらに serialize / deserialize の両 doctest は checks_print_report を呼ぶが stdout と exit_code metadata を固定していない。

## 対象

- `tests/stdlib/traits_serde.n.md`

## 根拠

- `serialize_trait_for_primitives` は `<i64> cast 9001` を使うが、`core/cast` を import していなかった。
- focused run では `resolve.identifier.undefined` と `type.annotation.mismatch` で serialize doctest が compile failure になっていた。
- serialize / deserialize の両方とも `checks_print_report` を呼んでいたが、manifest に `stdio` / `normalize_newlines` / `stdout:` / `exit_code:` がなかった。

## 問題

tests/stdlib/traits_serde.n.md の serialize doctest は core/cast を import せず cast を使って compile failure になっている。さらに serialize / deserialize の両 doctest は checks_print_report を呼ぶが stdout と exit_code metadata を固定していない。

## 影響

Serialize / Deserialize abstraction の重要な回帰テストが実行不能または report format 未検査になり、trait 抽象化の退行を CI fixture 差分として追えない。

## 修正方針

serialize doctest に core/cast import を追加し、2 doctest を stdio + normalize_newlines + exit_code: 0 + deterministic stdout fixture に移行する。source policy regression で import drift と stdout 欠落を拒否する。

## 検証

node nodesrc/test_stdlib_traits_serde_report_contract.js; node nodesrc/tests.js -i tests/stdlib/traits_serde.n.md --no-tree -o tmp/agent1-traits-serde-report.json -j 1 --dist web/dist --assert-io

## 2026-05-18 修正

`serialize_trait_for_primitives` に `#import "core/cast" as *` を追加し、`cast` を使う fixture が現行 import discipline に従うようにした。

2 doctest を `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` fixture に移行した。

- Serialize primitive report: 4 assertions
- Deserialize primitive / parse error report: 3 assertions

`nodesrc/test_stdlib_traits_serde_report_contract.js` を追加し、`ret:` 代用、stdout 欠落、serialize doctest の `core/cast` import 欠落へ戻らないことを source policy regression に登録した。
