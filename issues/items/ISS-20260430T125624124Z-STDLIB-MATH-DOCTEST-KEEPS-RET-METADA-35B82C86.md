---
id: ISS-20260430T125624124Z-STDLIB-MATH-DOCTEST-KEEPS-RET-METADA-35B82C86
title: "stdlib math doctest keeps ret metadata despite stdout report"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-04-30
updated: 2026-04-30
target: stdlib/tests/math.n.md
source: issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md
---

# ISS-20260430T125624124Z-STDLIB-MATH-DOCTEST-KEEPS-RET-METADA-35B82C86: stdlib math doctest keeps ret metadata despite stdout report

## 概要

stdlib/tests/math.n.md already prints a std/test assertion report, but the doctest metadata still uses ret: 0 and does not pin stdout.

## 対象

- `stdlib/tests/math.n.md`

## 根拠

- `stdlib/tests/math.n.md` は `checks_print_report checks` を既に呼んでいた。
- しかし metadata は `ret: 0` のままで、stdout expectation もなかった。
- そのため assertion report format が変わっても `.n.md` fixture で比較できず、exit contract も `ret:` に残っていた。

## 問題

stdlib/tests/math.n.md already prints a std/test assertion report, but the doctest metadata still uses ret: 0 and does not pin stdout.

## 影響

The runner contract remains ambiguous: process success is described as a return value, and the deterministic math assertion report can regress without stdout comparison.

## 修正方針

Replace ret: 0 with exit_code: 0 and add the deterministic stdout report expected from checks_print_report.

## 検証

Run node nodesrc/tests.js -i stdlib/tests/math.n.md --no-tree -o tmp/stdlib-math-report-agent1.json -j 1 --dist web/dist, node nodesrc/issues.js check, and git diff --check.

## 対応結果

`stdlib/tests/math.n.md` の metadata を `exit_code: 0` へ移行し、`checks_print_report` が出す27件の assertion reportを stdout fixture として固定した。

検証:

- `node nodesrc/tests.js -i stdlib/tests/math.n.md --no-tree -o tmp/stdlib-math-report-agent1.json -j 1 --dist web/dist`: total=1, passed=1
