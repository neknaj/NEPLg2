---
id: ISS-20260430T131836940Z-STDLIB-VEC-DOCTESTS-STILL-EXCEED-RUN-44D56F6B
title: "stdlib vec doctests still exceed runner budget and keep ret metadata"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-04-30
updated: 2026-04-30
target: stdlib/tests/vec.n.md
source: issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md
---

# ISS-20260430T131836940Z-STDLIB-VEC-DOCTESTS-STILL-EXCEED-RUN-44D56F6B: stdlib vec doctests still exceed runner budget and keep ret metadata

## 概要

stdlib/tests/vec.n.md still has monolithic std/test doctests that exceed the aggregate runner 60s case budget, and the same fixtures keep ret: metadata instead of stdout-pinned exit_code contracts.

## 対象

- `stdlib/tests/vec.n.md`

## 根拠

- `node nodesrc/tests.js -i stdlib/tests/vec.n.md --no-tree -o tmp/stdlib-vec-report-before-agent1.json -j 1 --dist web/dist` は `doctest#2` / `doctest#3` が `wasm test case timeout after 60000ms` で partial になった。
- focused `run_doctest` では `doctest#2` が約127秒、`doctest#3` が約68秒かかり、aggregate runner の per-case budget を超えていた。
- 同じ fixture は `checks_print_report` を呼ぶ一方で `ret: 0` のままで、stdout expectation もなかった。

## 問題

stdlib/tests/vec.n.md still has monolithic std/test doctests that exceed the aggregate runner 60s case budget, and the same fixtures keep ret: metadata instead of stdout-pinned exit_code contracts.

## 影響

Vec regressions cannot be checked reliably in the aggregate runner, and assertion report formatting can regress without fixture comparison.

## 修正方針

Split the large Vec doctests into smaller focused cases, migrate them to exit_code metadata, and pin each std/test stdout report.

## 検証

Run node nodesrc/tests.js -i stdlib/tests/vec.n.md --no-tree -o tmp/stdlib-vec-report-agent1.json -j 1 --dist web/dist, node nodesrc/issues.js check, and git diff --check.

## 対応結果

`stdlib/tests/vec.n.md` の大きい doctest を10件の focused doctest へ分割し、各 case を `exit_code: 0` + `stdout: mlstr` の `std/test` assertion report fixture に移行した。

分割後の aggregate runner では全 case が60秒未満で完了し、`ret:` と `checks_exit_code checks` の残存もなくなった。

検証:

- `node nodesrc/tests.js -i stdlib/tests/vec.n.md --no-tree -o tmp/stdlib-vec-report-agent1.json -j 1 --dist web/dist`: total=10, passed=10
- 各 case duration: 22.9秒から34.0秒
