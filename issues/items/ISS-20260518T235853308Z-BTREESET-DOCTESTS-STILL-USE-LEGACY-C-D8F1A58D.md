---
id: ISS-20260518T235853308Z-BTREESET-DOCTESTS-STILL-USE-LEGACY-C-D8F1A58D
title: "BTreeSet doctests still use legacy checks reports without stdout fixtures"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-18
updated: 2026-05-18
target: "stdlib/tests/btreeset.n.md, nodesrc/test_stdlib_btreeset_report_contract.js, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md"
---

# ISS-20260518T235853308Z-BTREESET-DOCTESTS-STILL-USE-LEGACY-C-D8F1A58D: BTreeSet doctests still use legacy checks reports without stdout fixtures

## 概要

stdlib/tests/btreeset.n.md has five runtime doctests that call checks_print_report and checks_exit_code, but the manifests do not pin stdout / exit_code metadata and the code still uses legacy checks_* helpers.

## 対象

- `stdlib/tests/btreeset.n.md, nodesrc/test_stdlib_btreeset_report_contract.js, issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md`

## 根拠

- `stdlib/tests/btreeset.n.md` の 5 doctest は、すべて `checks_print_report` と `checks_exit_code` を呼んでいた。
- しかし manifest は `neplg2:test` のままで、`stdio` / `normalize_newlines` / `stdout:` / `exit_code:` を固定していなかった。
- `stdlib/tests/btreemap.n.md` は already canonical `TestReport` contract に移行済みであり、BTreeSet 側だけが同じ collection family の中で旧 `checks_*` 形式を残していた。

## 問題

stdlib/tests/btreeset.n.md has five runtime doctests that call checks_print_report and checks_exit_code, but the manifests do not pin stdout / exit_code metadata and the code still uses legacy checks_* helpers.

## 影響

BTreeSet insert, growth, remove, duplicate insert, and borrowed observer behavior can regress without fixture-checked assertion labels and expected/actual stdout details.

## 修正方針

Migrate all five BTreeSet doctests to named TestReport stdout fixtures, add exit_code metadata, and add a source policy contract mirroring the existing BTreeMap report contract.

## 検証

Run the BTreeSet source policy contract, focused BTreeSet doctests, source policy regressions, issue check, and diff whitespace check.

## 対応結果

- `stdlib/tests/btreeset.n.md` の 5 doctest を `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` fixture へ移行した。
- 旧 `checks_new` / `checks_push` / `checks_print_report` / `checks_exit_code` を named `TestReport` API へ置き換えた。
- insert / grow / remove / duplicate insert / borrowed observer の各観測値を assertion label と expected / actual 付きで stdout に固定した。
- `nodesrc/test_stdlib_btreeset_report_contract.js` を追加し、`ret:` 代用、stdout fixture 欠落、旧 `checks_*` 形式への退行を source policy で拒否する。

## 関連

- [`.n.md tests rely on return values instead of stdout assertion reports`](./ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md)
