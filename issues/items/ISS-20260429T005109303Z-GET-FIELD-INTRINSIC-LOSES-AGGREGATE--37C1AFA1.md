---
id: ISS-20260429T005109303Z-GET-FIELD-INTRINSIC-LOSES-AGGREGATE--37C1AFA1
title: "get_field intrinsic loses aggregate field function aliases in move_check"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core/src/passes/move_check/alias.rs, nepl-core/src/typecheck/field_apply.rs, tests/compiler/move_effect.n.md"
---

# ISS-20260429T005109303Z-GET-FIELD-INTRINSIC-LOSES-AGGREGATE--37C1AFA1: get_field intrinsic loses aggregate field function aliases in move_check

## 概要

After field::get/get_field started preserving selector identity in HIR, move_check function alias recovery still only treated Call get and raw load(add(...)) as field projections. Function values stored in aggregate fields and read by intrinsic get_field therefore lost their known callee alias, so aggregate-field callback raw writes could bypass raw memory effect checking.

## 対象

- `nepl-core/src/passes/move_check/alias.rs, nepl-core/src/typecheck/field_apply.rs, tests/compiler/move_effect.n.md`

## 根拠

- `ISS-20260428T233410073Z-GENERIC-OWNED-AGGREGATE-FIELD-MOVES--0A6FA87B` の修正で、field selector identity を保持するため `field::get` / `get_field` は HIR 上に intrinsic `get_field(base, selector)` として残るようになった。
- その後 `node nodesrc\tests.js -i tests\compiler\move_effect.n.md --no-tree -o tmp\agent1-resource-borrow-gate-move-effect.json -j 1` を実行すると、`doctest#69` が `expected compile_fail, but compiled successfully` になった。
- 該当 fixture は `CallbackHolder.cb` に格納した `@clobber_i32` を `field::get holder "cb"` で取り出して呼び出す。move_check の function alias recovery が intrinsic `get_field` を field projection として扱っていなかったため、known callee の raw memory effect summary が `CallIndirect` に適用されなかった。

## 問題

After field::get/get_field started preserving selector identity in HIR, move_check function alias recovery still only treated Call get and raw load(add(...)) as field projections. Function values stored in aggregate fields and read by intrinsic get_field therefore lost their known callee alias, so aggregate-field callback raw writes could bypass raw memory effect checking.

## 影響

A CallbackHolder field containing @clobber_i32 could be read with field::get and invoked without applying the callee raw memory effect summary, letting initialized non-Copy raw cells be overwritten without D3100.

## 修正方針

Teach move_check field_get_projection and function/aggregate field alias recovery to handle intrinsic get_field the same way as source-level field::get calls, while preserving the existing raw load fallback.

## 検証

- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc\tests.js -i tests\compiler\move_effect.n.md --no-tree -o tmp\agent1-resource-borrow-gate-alias-move-effect.json -j 1`: total=110, passed=110, failed=0
- `node nodesrc\tests.js -i tests\compiler\move_check.n.md --no-tree -o tmp\agent1-resource-borrow-gate-alias-move-check.json -j 1`: total=52, passed=52, failed=0

## 対応結果

`field_get_projection` が source-level `get` call だけでなく intrinsic `get_field` も同じ projection として返すようにした。あわせて function value alias / aggregate field function alias / aggregate raw alias の復元で `get_field` projection を raw `load` fallback より先に扱うようにし、selector-preserving HIR と move_check の alias recovery を整合させた。

関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-%E3%81%B8%E3%81%AE%E7%A7%BB%E8%A1%8C)
