---
id: ISS-20260507T005013028Z-MEMORY-SAFETY-FIXTURES-STILL-CALL-UN-3C928E60
title: "memory_safety fixtures still call unsafe memory from pure main"
area: TEST
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-07
updated: 2026-05-07
target: tests/stdlib/memory_safety.n.md
---

# ISS-20260507T005013028Z-MEMORY-SAFETY-FIXTURES-STILL-CALL-UN-3C928E60: memory_safety fixtures still call unsafe memory from pure main

## 概要

tests/stdlib/memory_safety.n.md keeps runtime memory-operation doctests as pure main functions after Stage 5 ResourceIR unsafe-memory diagnostics became authoritative. The compiler correctly emits effect.pure.calls_impure, but the stale fixtures report this as runtime test failures.

## 対象

- `tests/stdlib/memory_safety.n.md`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 5 は `UnsafeMemoryInPureFunction` を Resource IR diagnostic から compiler error へ接続済みであり、pure source からの raw memory operation は `effect.pure.calls_impure` として拒否する方針である。
- 修正前の `tests/stdlib/memory_safety.n.md` は、`alloc_ptr` / `load_i32` / `store_i32` / `fill_*` など raw memory backed helper の runtime behavior fixture をすべて pure `main <()->i32>` で実行していた。
- focused 実行では 12 件中 7 件が compile phase で `effect.pure.calls_impure` になった。これは gate の誤検出ではなく、fixture が unsafe memory boundary を明示していないことが原因だった。

## 問題

tests/stdlib/memory_safety.n.md keeps runtime memory-operation doctests as pure main functions after Stage 5 ResourceIR unsafe-memory diagnostics became authoritative. The compiler correctly emits effect.pure.calls_impure, but the stale fixtures report this as runtime test failures.

## 影響

The focused memory safety suite remains red even though the static checker is enforcing the intended boundary. Future work may be misled into weakening ResourceIR effect diagnostics instead of preserving the pure/impure boundary.

## 修正方針

Move runtime raw-memory behavior fixtures to explicit impure entry functions and add pure compile_fail coverage that locks direct unsafe memory operations to effect.pure.calls_impure.

## 検証

Run node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree --dist web/dist -j 1 --assert-io and require all tests to pass.

## 修正内容

- raw memory wrapper の runtime behavior を確認する doctest は `main <()*>i32` へ移行し、unsafe memory operation を明示的に impure boundary 内で実行するようにした。
- pure 文脈から `MemPtr` overload の `load_i32` / `store_i32` / `fill_u8` を呼ぶ compile_fail fixture を追加し、`effect.pure.calls_impure` を regression として固定した。
- compiler 側の Resource IR effect diagnostic は弱めず、Stage 5 の pure boundary enforcement を正しい authority として維持した。

## 検証結果

- `NEPL_TEST_CASE_TIMEOUT_MS=120000 node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree --dist web/dist -o tmp/memory_safety_agent1_after_fixture_update.json -j 1 --assert-io`: total=14, passed=14, failed=0, errored=0

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
