---
id: ISS-20260514T030107222Z-OVERLOAD-GENERIC-VEC-HELPER-LACKS-CO-87D93F09
title: "overload generic Vec helper lacks Copy bound after Vec Copy-only boundary"
area: TEST
status: open
resolved: false
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

- 未記入

## 問題

tests/compiler/overload.n.md::doctest#10 の pair_with_empty<.T> が v::new<.T> を呼ぶが、現行 Vec API は transitional collection boundary として .T: Copy を要求するため type.trait_bound.unsatisfied で失敗する。

## 影響

overload / tuple field regression の full focused run が未変更 fixture で失敗し、Vec Copy-only boundary と generic helper の契約がずれている。

## 修正方針

pair_with_empty に .T: Copy bound を明示し、i32 具体化 case の検査意図を維持する。必要なら stdout report 移行時にこの境界を assertion label として固定する。

## 検証

node nodesrc/run_doctest.js -i tests/compiler/overload.n.md -n 10 --assert-io --dist web/dist
