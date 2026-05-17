---
id: ISS-20260517T040636489Z-MEMORY-SAFETY-OWNER-AGGREGATE-FIXTUR-968B23FB
title: "memory_safety owner aggregate fixtures still use old Vec layout"
area: TEST
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-17
updated: 2026-05-17
target: tests/stdlib/memory_safety.n.md
---

# ISS-20260517T040636489Z-MEMORY-SAFETY-OWNER-AGGREGATE-FIXTUR-968B23FB: memory_safety owner aggregate fixtures still use old Vec layout

## 概要

memory_safety doctest#35/#36 expect owner aggregate constructor/field diagnostics, but the fixture still uses the pre-OwnedBuffer Vec layout, so current compiler reports overload/field-shape errors before exercising the static-check boundary.

## 対象

- `tests/stdlib/memory_safety.n.md`

## 根拠

- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/memory-safety-stage6-current.json -j 1`: total=41, passed=39, failed=2。
- doctest#35 は旧 `Vec<i32> 0 1 ...` layout のため `type.overload.no_match` / `type.annotation.mismatch` になり、`type.owner_aggregate.constructor_restricted` に到達していなかった。
- doctest#36 は旧 `Vec.storage` field を参照していたため `type.field.invalid_access` になり、`type.owner_aggregate.field_access_restricted` に到達していなかった。

## 問題

memory_safety doctest#35/#36 expect owner aggregate constructor/field diagnostics, but the fixture still uses the pre-OwnedBuffer Vec layout, so current compiler reports overload/field-shape errors before exercising the static-check boundary.

## 影響

The regression suite no longer proves that owner aggregate constructor and field access restrictions still fire for the current Vec/OwnedBuffer design.

## 修正方針

Update the fixtures to use the current Vec(OwnedBuffer(...)) layout and Vec.buffer field boundary so the intended static-check diagnostics are exercised directly.

## 検証

- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/memory-safety-ownedbuffer-regression.json -j 1`: 41/41 passed

## 2026-05-17 Agent 1 修正結果

- owner aggregate constructor regression は、現在の `Vec<i32>` layout に合わせて `Vec<i32> (OwnedBuffer<i32> 0 1 (VecStorage<i32>::Owned region))` を使うように更新した。
- owner aggregate field access regression は、削除済みの `Vec.storage` ではなく現在の `Vec.buffer` field を `field::get_ref` で直接読む fixture に更新した。
- 期待診断は `type.owner_aggregate.constructor_restricted` / `type.owner_aggregate.field_access_restricted` のまま維持し、静的検査の境界を弱めていない。
