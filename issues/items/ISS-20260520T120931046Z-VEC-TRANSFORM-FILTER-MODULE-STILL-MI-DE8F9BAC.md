---
id: ISS-20260520T120931046Z-VEC-TRANSFORM-FILTER-MODULE-STILL-MI-DE8F9BAC
title: "Vec transform filter module still mixes filter and partition responsibilities"
area: stdlib
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/alloc/collections/vec/transform/filter.nepl, stdlib/alloc/collections/vec/transform/filter/select.nepl, stdlib/alloc/collections/vec/transform/filter/partition.nepl, stdlib/alloc/collections/vec/transform/filter/partition/build.nepl, stdlib/alloc/collections/vec/transform/filter/partition/view.nepl, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js"
---

# ISS-20260520T120931046Z-VEC-TRANSFORM-FILTER-MODULE-STILL-MI-DE8F9BAC: Vec transform filter module still mixes filter and partition responsibilities

## 概要

Vec transform/filter.nepl owns both filter and partition plus VecPartition observer/free APIs, so predicate selection, two-way partition construction, and raw storage boundary checks are still coupled in one large implementation file.

## 対象

- `stdlib/alloc/collections/vec/transform/filter.nepl, stdlib/alloc/collections/vec/transform/filter/select.nepl, stdlib/alloc/collections/vec/transform/filter/partition.nepl, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`

## 根拠

- 未記入

## 問題

Vec transform/filter.nepl owns both filter and partition plus VecPartition observer/free APIs, so predicate selection, two-way partition construction, and raw storage boundary checks are still coupled in one large implementation file.

## 影響

The file remains one of the largest stdlib files, makes Vec raw-memory safety policy harder to audit, and increases the risk that future static-check fixes reintroduce implementation bodies into a facade or broaden raw-memory capability.

## 修正方針

Turn transform/filter.nepl into a pure public facade. Move filter into transform/filter/select.nepl. Turn transform/filter/partition.nepl into a partition-family facade, move the raw-storage partition construction into partition/build.nepl, and move VecPartition observer/free APIs into partition/view.nepl. Update source policy to lock the facade/submodule boundary and exact raw-memory evidence paths.

## 検証

Run Vec source policy plus focused doctests for the filter facade and both implementation submodules.

## 対応内容

- `stdlib/alloc/collections/vec/transform/filter.nepl` を pure facade にし、`filter` と `partition` family を public re-export するだけにした。
- `filter` 本体を `stdlib/alloc/collections/vec/transform/filter/select.nepl` へ移し、単一出力の predicate selection に必要な raw storage write だけを閉じ込めた。
- `partition` family は `stdlib/alloc/collections/vec/transform/filter/partition.nepl` を facade にし、2 出力 owner construction を `partition/build.nepl`、observer/free を `partition/view.nepl` へ分離した。
- `partition/view.nepl` は raw memory operation を直接持たず、`VecPartition` の field projection と cleanup API だけを所有する。これにより raw memory boundary capability を持つ file と public observer/free を分けた。
- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` と `nodesrc/test_stdlib_vec_borrowed_observers.js` を更新し、facade に実装本体が戻らないこと、raw memory evidence が `select.nepl` / `partition/build.nepl` に限定されること、observer/free が `partition/view.nepl` に留まることを固定した。

## 検証結果

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_vec_borrowed_observers.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/transform/filter.nepl --no-tree --dist web/dist -o tmp/agent1-vec-transform-filter-split-facade-2.json -j 1 --assert-io`: total=1, passed=1
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/transform/filter/select.nepl --no-tree --dist web/dist -o tmp/agent1-vec-transform-filter-split-select-2.json -j 1 --assert-io`: total=1, passed=1
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/transform/filter/partition.nepl -i stdlib/alloc/collections/vec/transform/filter/partition/build.nepl -i stdlib/alloc/collections/vec/transform/filter/partition/view.nepl --no-tree --dist web/dist -o tmp/agent1-vec-transform-filter-split-partition-2.json -j 1 --assert-io`: total=6, passed=6
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/transform.nepl -i stdlib/alloc/collections/vec/transform/map.nepl -i stdlib/alloc/collections/vec/transform/filter.nepl -i stdlib/alloc/collections/vec/transform/filter/select.nepl -i stdlib/alloc/collections/vec/transform/filter/partition.nepl -i stdlib/alloc/collections/vec/transform/filter/partition/build.nepl -i stdlib/alloc/collections/vec/transform/filter/partition/view.nepl -i stdlib/alloc/collections/vec/transform/prefix.nepl --no-tree --dist web/dist -o tmp/agent1-vec-transform-filter-split-transform-suite-2.json -j 1 --assert-io`: total=11, passed=11
