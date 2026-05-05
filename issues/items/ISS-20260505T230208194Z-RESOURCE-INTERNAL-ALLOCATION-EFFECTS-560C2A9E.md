---
id: ISS-20260505T230208194Z-RESOURCE-INTERNAL-ALLOCATION-EFFECTS-560C2A9E
title: "Resource internal allocation effects drop raw operation identity"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-05
updated: 2026-05-06
target: "nepl-core/src/resource/model.rs, nepl-core/src/resource/lower.rs, nepl-core/src/resource/effect_check.rs"
---

# ISS-20260505T230208194Z-RESOURCE-INTERNAL-ALLOCATION-EFFECTS-560C2A9E: Resource internal allocation effects drop raw operation identity

## 概要

InternalEffect::InternalAlloc keeps the typed RawMemoryOp, but resource lowering collapses it to EffectOp::InternalAlloc without operation detail. Resource IR effect summaries can count internal allocations but cannot distinguish alloc, dealloc, realloc, memory.size, and memory.grow with exhaustive matches.

## 対象

- `nepl-core/src/resource/model.rs, nepl-core/src/resource/lower.rs, nepl-core/src/resource/effect_check.rs`

## 根拠

- `InternalEffect::InternalAlloc` は `RawMemoryOp` を保持しているが、`resource_effect_from_internal` は `EffectOp::InternalAlloc` へ畳み込み、operation を失っていた。
- `EffectOp::UnsafeMemory` は `RawMemoryOp` を保持するよう修正済みであり、`InternalAlloc` だけが Stage 5 effect model の enum-first 方針から外れていた。
- Resource IR dump も `effect internal_alloc` までしか出せず、`alloc` / `dealloc` / `realloc` / `memory_size` / `memory_grow` を後続検査や review で区別しにくかった。

## 問題

InternalEffect::InternalAlloc keeps the typed RawMemoryOp, but resource lowering collapses it to EffectOp::InternalAlloc without operation detail. Resource IR effect summaries can count internal allocations but cannot distinguish alloc, dealloc, realloc, memory.size, and memory.grow with exhaustive matches.

## 影響

Stage 5 effect checking depends on enum-first effect operations. Dropping internal allocation operation identity weakens diagnostics and makes later raw memory boundary enforcement harder to audit.

## 修正方針

Carry RawMemoryOp through EffectOp::InternalAlloc and update Resource IR dump/checkers/tests to match on the typed operation.

## 対応

- `EffectOp::InternalAlloc` を `EffectOp::InternalAlloc { operation: RawMemoryOp }` に変更した。
- `resource_effect_from_internal` で `InternalEffect::InternalAlloc` の operation を Resource IR へそのまま渡すようにした。
- downstream checker の match を `InternalAlloc { .. }` に更新し、enum variant 追加時に compiler の網羅性検査が効く形を維持した。
- Resource IR dump を `internal_alloc(alloc)` のように operation 付きで出すようにし、`resource_ir_lowering_preserves_raw_memory_operations` で固定した。
- 親 issue [ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04](./ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04.md) の Stage 5 effect model 進捗として扱う。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_lowering_preserves_raw_memory_operations -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 155 passed
- `cargo test -p nepl-core --test effects -- --nocapture`: 23 passed
