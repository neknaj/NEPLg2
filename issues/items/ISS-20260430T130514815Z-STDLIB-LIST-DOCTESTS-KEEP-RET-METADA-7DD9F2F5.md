---
id: ISS-20260430T130514815Z-STDLIB-LIST-DOCTESTS-KEEP-RET-METADA-7DD9F2F5
title: "stdlib list doctests keep ret metadata despite stdout reports"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-04-30
updated: 2026-04-30
target: stdlib/tests/list.n.md
source: issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md
---

# ISS-20260430T130514815Z-STDLIB-LIST-DOCTESTS-KEEP-RET-METADA-7DD9F2F5: stdlib list doctests keep ret metadata despite stdout reports

## 概要

stdlib/tests/list.n.md prints std/test assertion reports, but both doctest manifests still use ret: 0 and do not pin stdout.

## 対象

- `stdlib/tests/list.n.md`

## 根拠

- `stdlib/tests/list.n.md` は2件の doctest で `checks_print_report checks` を既に呼んでいる。
- しかし metadata はどちらも `ret: 0` のままで、stdout expectation もなかった。
- そのため assertion report format が変わっても `.n.md` fixture で比較できず、exit contract も `ret:` に残っていた。

## 問題

stdlib/tests/list.n.md prints std/test assertion reports, but both doctest manifests still use ret: 0 and do not pin stdout.

## 影響

The list assertion reports can regress without fixture comparison, and these tests still use return-value metadata as process success.

## 修正方針

Replace ret: 0 with exit_code: 0 and add deterministic stdout expectations for both checks_print_report outputs.

## 検証

Run node nodesrc/tests.js -i stdlib/tests/list.n.md --no-tree -o tmp/stdlib-list-report-agent1.json -j 1 --dist web/dist, node nodesrc/issues.js check, and git diff --check.

## 対応結果

`stdlib/tests/list.n.md` の2件の doctest metadata を `exit_code: 0` へ移行し、`checks_print_report` が出す14件と10件の assertion reportを stdout fixture として固定した。

検証:

- `node nodesrc/tests.js -i stdlib/tests/list.n.md --no-tree -o tmp/stdlib-list-report-agent1.json -j 1 --dist web/dist`: total=2, passed=2
