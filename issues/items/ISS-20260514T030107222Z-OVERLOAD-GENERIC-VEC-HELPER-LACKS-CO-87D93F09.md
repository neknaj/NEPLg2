---
id: ISS-20260514T030107222Z-OVERLOAD-GENERIC-VEC-HELPER-LACKS-CO-87D93F09
title: "overload generic Vec helper lacks Copy bound after Vec Copy-only boundary"
area: TEST
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-14
target: tests/compiler/overload.n.md
---

# ISS-20260514T030107222Z-OVERLOAD-GENERIC-VEC-HELPER-LACKS-CO-87D93F09: overload generic Vec helper lacks Copy bound after Vec Copy-only boundary

## 概要

tests/compiler/overload.n.md::doctest#10 の pair_with_empty<.T> が v::new<.T> を呼ぶが、現行 Vec API は transitional collection boundary として .T: Copy を要求するため type.trait_bound.unsatisfied で失敗する。

## 対象

- `tests/compiler/overload.n.md`

## 根拠

- `v::new<.T>` は現行の `Vec` API で `.T: Copy` を要求する。
- `pair_with_empty<.T>` は generic helper の本体で `v::new<.T>` を呼ぶため、関数定義側に同じ境界を持たなければならない。
- 具体化が `i32` で成立していることに依存すると、generic body の契約が曖昧になり、trait bound 検査の回帰 fixture として不適切になる。

## 問題

tests/compiler/overload.n.md::doctest#10 の pair_with_empty<.T> が v::new<.T> を呼ぶが、現行 Vec API は transitional collection boundary として .T: Copy を要求するため type.trait_bound.unsatisfied で失敗する。

## 影響

overload / tuple field regression の full focused run が未変更 fixture で失敗し、Vec Copy-only boundary と generic helper の契約がずれている。

## 修正方針

pair_with_empty に .T: Copy bound を明示し、i32 具体化 case の検査意図を維持する。必要なら stdout report 移行時にこの境界を assertion label として固定する。

## 対応結果

- `pair_with_empty<.T>` を `pair_with_empty<.T: Copy>` に変更し、`Vec<.T>` の生成に必要な境界を helper 自身の型契約として明示した。
- `Vec` 側の Copy-only boundary や compiler の trait bound 検査は緩めていない。
- `overload_pair_field_from_generic_result_keeps_tuple_type` は `i32` 具体化の成功経路を維持しつつ、generic helper の契約不備を隠さない形になった。

## 検証

- `node nodesrc/run_doctest.js -i tests/compiler/overload.n.md -n 10 --assert-io --dist web/dist`: pass
- `node nodesrc/tests.js -i tests/compiler/overload.n.md --no-tree -o tmp/agent1-overload-generic-vec-copy-bound.json -j 1 --assert-io --dist web/dist`: total=45, passed=45, failed=0
