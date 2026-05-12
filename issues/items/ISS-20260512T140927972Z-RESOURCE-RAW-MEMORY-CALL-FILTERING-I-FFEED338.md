---
id: ISS-20260512T140927972Z-RESOURCE-RAW-MEMORY-CALL-FILTERING-I-FFEED338
title: "Resource raw memory call filtering is duplicated between lowering and coverage"
area: core
status: fixed
resolved: true
priority: P1
type: maintenance
created: 2026-05-12
updated: 2026-05-12
target: "nepl-core/src/resource/lower.rs; nepl-core/src/resource/coverage_hir_raw.rs; nepl-core/src/resource/lower_raw_memory.rs"
---

# ISS-20260512T140927972Z-RESOURCE-RAW-MEMORY-CALL-FILTERING-I-FFEED338: Resource raw memory call filtering is duplicated between lowering and coverage

## 概要

Resource IR lowering and HIR coverage both repeat the same RawMemoryOp argument filter to avoid treating MemPtr overload wrappers as direct raw memory operations.

## 対象

- `nepl-core/src/resource/lower.rs; nepl-core/src/resource/coverage_hir_raw.rs; nepl-core/src/resource/lower_raw_memory.rs`

## 根拠

- `nepl-core/src/resource/lower.rs` は `RawMemoryOp` call を Resource IR `RawMemory` op へ下げる時点で MemPtr wrapper overload を除外していた。
- `nepl-core/src/resource/coverage_hir_raw.rs` は HIR 側の raw memory coverage count で同じ除外条件を別実装していた。
- Stage 3 の lowering / coverage agreement と Stage 5 の raw memory operation authority は、同じ enum branch を複数箇所へ複製すると追加 variant 時に崩れる。

## 問題

Resource IR lowering and HIR coverage both repeat the same RawMemoryOp argument filter to avoid treating MemPtr overload wrappers as direct raw memory operations.

## 影響

A future RawMemoryOp or MemPtr wrapper change can make Resource IR lowering and coverage count different operation sets, weakening the coverage gate or creating false coverage failures.

## 修正方針

Move the raw memory call argument filter to lower_raw_memory.rs and make both lowering and coverage consume the same typed helper.

## 対応記録

- `nepl-core/src/resource/lower_raw_memory.rs` に `raw_memory_call_uses_direct_raw_address` を追加し、`RawMemoryOp` ごとの direct raw-address requirement を単一の typed helper に集約した。
- Resource IR lowering と HIR coverage count の双方をこの helper 経由へ変更した。
- `nodesrc/test_resource_checker_responsibility.js` に、`lower.rs` 側の local filter と coverage 側の direct `is_named_struct_type` 判定を再導入しない source policy を追加した。

## 検証

- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_lowering_preserves_raw_memory_operations -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_lowering_coverage_guards_borrow_and_deref_places -- --nocapture`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo fmt --check -p nepl-core`: passed
