---
id: ISS-20260430T132753960Z-STDLIB-HASHSET-DOCTESTS-KEEP-RET-MET-4238B95D
title: "stdlib hashset doctests keep ret metadata without pinned reports"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-04-30
updated: 2026-04-30
target: stdlib/tests/hashset.n.md
source: issues/items/ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD.md
---

# ISS-20260430T132753960Z-STDLIB-HASHSET-DOCTESTS-KEEP-RET-MET-4238B95D: stdlib hashset doctests keep ret metadata without pinned reports

## 概要

stdlib/tests/hashset.n.md uses ret: metadata for process success; the main std/test case prints a report without pinning stdout, and the free smoke case has no stdout assertion contract.

## 対象

- `stdlib/tests/hashset.n.md`

## 根拠

- `stdlib/tests/hashset.n.md::doctest#1` は `checks_print_report checks` を呼んでいたが、metadata は `ret: 0` のままで stdout expectation がなかった。
- `hashset_free_smoke` は成功を `ret: 0` だけで表しており、free が成功したことを stdout 上の assertion report として確認できなかった。

## 問題

stdlib/tests/hashset.n.md uses ret: metadata for process success; the main std/test case prints a report without pinning stdout, and the free smoke case has no stdout assertion contract.

## 影響

HashSet assertion report formatting can regress without fixture comparison, and the smoke case still uses return-value metadata as process success.

## 修正方針

Replace ret metadata with exit_code, pin the existing std/test report, and make the smoke case emit a minimal std/test report after free succeeds.

## 検証

Run node nodesrc/tests.js -i stdlib/tests/hashset.n.md --no-tree -o tmp/stdlib-hashset-report-agent1.json -j 1 --dist web/dist, node nodesrc/issues.js check, and git diff --check.

## 対応結果

`stdlib/tests/hashset.n.md` の2件の doctest metadata を `exit_code: 0` へ移行した。main case は既存の8件 reportを stdout fixture として固定し、free smoke case は free 後に最小の `std/test` report を出す形へ揃えた。

検証:

- `node nodesrc/tests.js -i stdlib/tests/hashset.n.md --no-tree -o tmp/stdlib-hashset-report-agent1.json -j 1 --dist web/dist`: total=2, passed=2
