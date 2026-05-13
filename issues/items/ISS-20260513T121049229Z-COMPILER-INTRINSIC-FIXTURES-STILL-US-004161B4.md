---
id: ISS-20260513T121049229Z-COMPILER-INTRINSIC-FIXTURES-STILL-US-004161B4
title: "compiler intrinsic fixtures still use pure raw-memory entries"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-13
updated: 2026-05-13
target: tests/compiler/intrinsic.n.md
---

# ISS-20260513T121049229Z-COMPILER-INTRINSIC-FIXTURES-STILL-US-004161B4: compiler intrinsic fixtures still use pure raw-memory entries

## 概要

tests/compiler/intrinsic.n.md still runs raw load/store/dealloc intrinsic behavior tests through pure main functions and omits the current core/mem import for size_of/align_of. Current Resource IR effect checking correctly reports effect.pure.calls_impure and resolve.identifier.undefined, so the fixture is stale rather than the checker being too strict.

## 対象

- `tests/compiler/intrinsic.n.md`

## 根拠

- `node nodesrc/tests.js -i tests/compiler/intrinsic.n.md --no-tree -o tmp/agent1-intrinsic-current.json -j 1 --dist web/dist` で、raw `store` / `load` / `dealloc` を含む runtime fixtures が `effect.pure.calls_impure` で失敗した。
- `intrinsic_size_of_std_layout` は `size_of` / `align_of` を `core/mem` から import していないため、現在の layout API 境界では `resolve.identifier.undefined` になっていた。
- `effect.pure.calls_impure` は Resource IR の `UnsafeMemoryInPureFunction` から出る正しい hard error であり、raw memory fixture 側が impure entry を明示するべきである。

## 問題

tests/compiler/intrinsic.n.md still runs raw load/store/dealloc intrinsic behavior tests through pure main functions and omits the current core/mem import for size_of/align_of. Current Resource IR effect checking correctly reports effect.pure.calls_impure and resolve.identifier.undefined, so the fixture is stale rather than the checker being too strict.

## 影響

The compiler intrinsic doctest suite is red on current main and creates pressure to weaken Resource IR unsafe-memory effect diagnostics. It also fails to document the current public layout API import boundary.

## 修正方針

Update raw-memory runtime intrinsic tests to use explicit impure entry signatures, keep pure size/align tests on safe APIs, and import core/mem where size_of/align_of are used.

## 検証

Run node nodesrc/tests.js -i tests/compiler/intrinsic.n.md --no-tree -o tmp/agent1-intrinsic-fixtures-after.json -j 1 --dist web/dist and node nodesrc/issues.js check --dir issues.

## 2026-05-13 修正

`tests/compiler/intrinsic.n.md` の raw memory runtime fixtures を現在の effect model へ合わせた。

- `load` / `store` / `dealloc_raw` / `load_i32` / `store_i32` を直接呼ぶ runtime behavior tests は、entry signature を `fn main <()*>i32> ():` に変更した。
- raw memory を使わない `size_of` / `align_of` と unit enum payload の tests は pure のまま維持し、effect boundary を必要以上に広げていない。
- std layout test は `size_of` / `align_of` の現在の公開 API 境界である `core/mem` を直接 import するようにした。

検証:

- `node nodesrc/tests.js -i tests/compiler/intrinsic.n.md --no-tree -o tmp/agent1-intrinsic-fixtures-after.json -j 1 --dist web/dist`: total=8, passed=8
- `node nodesrc/issues.js check --dir issues`: passed
