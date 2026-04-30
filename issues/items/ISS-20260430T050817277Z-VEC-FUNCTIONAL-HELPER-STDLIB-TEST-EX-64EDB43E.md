---
id: ISS-20260430T050817277Z-VEC-FUNCTIONAL-HELPER-STDLIB-TEST-EX-64EDB43E
title: "Vec functional helper stdlib test exceeds doctest runtime budget"
area: stdlib
status: open
resolved: false
priority: P2
type: test
created: 2026-04-30
updated: 2026-04-30
target: stdlib/tests/vec.n.md
---

# ISS-20260430T050817277Z-VEC-FUNCTIONAL-HELPER-STDLIB-TEST-EX-64EDB43E: Vec functional helper stdlib test exceeds doctest runtime budget

## 概要

After Vec owner leaks were fixed, stdlib/tests/vec.n.md doctest#2 compiles but times out at the 60s wasm doctest limit. Focused probes for the same helper groups pass, so the residual problem is that one monolithic test does too much work in a single doctest.

## 対象

- `stdlib/tests/vec.n.md`

## 根拠

- `node nodesrc/tests.js -i stdlib/tests/vec.n.md --no-tree -o tmp/vec-stdlib-test-owner-discipline-after-fix.json -j 1 --dist web/dist` で、doctest#1 は pass したが doctest#2 が `wasm test case timeout after 60000ms` になった。
- 一時 probe で map/filter/fold/reduce 系、find/all/named result 系、partition/take/drop/count 系を分割して実行するとそれぞれ pass したため、単一機能の無限 loop ではなく 1 doctest の肥大化が主因と判断した。
- `stdlib/alloc/collections/vec.nepl` の module doctest 39 件と `tests/stdlib/vec_collections.n.md` は pass しており、Vec 実装修正の primary regression とは分離できる。

## 問題

After Vec owner leaks were fixed, stdlib/tests/vec.n.md doctest#2 compiles but times out at the 60s wasm doctest limit. Focused probes for the same helper groups pass, so the residual problem is that one monolithic test does too much work in a single doctest.

## 影響

The full Vec stdlib regression file still cannot be used as a clean CI signal even though the module doctests and focused collection tests pass. Future Vec changes may miss regressions if this timeout is treated as noise.

## 修正方針

Split vec_functional_helpers into smaller focused .n.md doctests or reduce the std/test aggregation overhead while keeping owner-safe cleanup in each block.

## 検証

Run stdlib/tests/vec.n.md to completion, Vec module doctests, and Vec collection tests.
