---
id: ISS-20260505T230807234Z-RESOURCE-EFFECT-COUNTS-COLLAPSE-TYPE-3CD39CCB
title: "Resource effect counts collapse typed raw operations"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-05
updated: 2026-05-06
target: "nepl-core/src/resource/effect.rs, nepl-core/src/resource/effect_check.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260505T230807234Z-RESOURCE-EFFECT-COUNTS-COLLAPSE-TYPE-3CD39CCB: Resource effect counts collapse typed raw operations

## 概要

Resource effect checks now carry typed RawMemoryOp for internal allocation and unsafe memory, but ResourceEffectCounts still stores only aggregate totals. This loses operation-level evidence for alloc/dealloc/realloc/load/store/fill/bulk operations in Stage 5 effect reports.

## 対象

- `nepl-core/src/resource/effect.rs, nepl-core/src/resource/effect_check.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `EffectOp::{InternalAlloc, UnsafeMemory}` は typed `RawMemoryOp` を保持するようになったが、`ResourceEffectCounts` は `internal_allocs` / `unsafe_memory_ops` の合計値だけを保持していた。
- `effect_check.rs` は operation を match せずに合計値だけを increment していたため、Stage 5 report から `alloc` / `load` / `store` / `fill` / bulk operation の内訳が消えていた。
- raw memory operation 追加時に count 側の更新漏れを compiler の exhaustiveness で検出できなかった。

## 問題

Resource effect checks now carry typed RawMemoryOp for internal allocation and unsafe memory, but ResourceEffectCounts still stores only aggregate totals. This loses operation-level evidence for alloc/dealloc/realloc/load/store/fill/bulk operations in Stage 5 effect reports.

## 影響

Static-check audits cannot tell which raw memory operations were observed from the effect report. This weakens review of raw memory boundary enforcement and makes operation-specific regressions harder to catch.

## 修正方針

Replace aggregate raw memory totals with RawMemoryEffectCounts that records each RawMemoryOp through exhaustive match, and update Resource IR tests to assert operation-specific counts.

## 対応

- `RawMemoryEffectCounts` を追加し、`RawMemoryOp` ごとの count を保持するようにした。
- `RawMemoryEffectCounts::record` は `RawMemoryOp` の全 variant を `match` で分岐し、新 operation 追加時の更新漏れを検出できる形にした。
- `ResourceEffectCounts` は internal memory と unsafe memory の両方で `RawMemoryEffectCounts` を保持するようにした。
- Resource IR の regression は `internal_memory_ops.alloc` と `unsafe_memory_ops.store` を直接確認するよう更新した。
- 親 issue [ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04](./ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04.md) の Stage 5 effect model 進捗として扱う。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_effect_check_reports_raw_alloc_return_escape -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_effect_check_reports_unsafe_memory_in_pure_function -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 155 passed
