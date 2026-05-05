---
id: ISS-20260505T225243749Z-RESOURCE-UNSAFE-MEMORY-EFFECTS-KEEP--D20087E5
title: "Resource unsafe memory effects keep raw operation as string"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-05
updated: 2026-05-06
target: "nepl-core/src/effects.rs, nepl-core/src/resource/model.rs, nepl-core/src/resource/lower_raw_memory.rs"
---

# ISS-20260505T225243749Z-RESOURCE-UNSAFE-MEMORY-EFFECTS-KEEP--D20087E5: Resource unsafe memory effects keep raw operation as string

## 概要

Resource IR has RawMemoryOp enum, but InternalEffect::UnsafeMemory and EffectOp::UnsafeMemory keep the raw memory operation as a String. This weakens exhaustiveness and lets new raw memory operations bypass compile-time match coverage.

## 対象

- `nepl-core/src/effects.rs, nepl-core/src/resource/model.rs, nepl-core/src/resource/lower_raw_memory.rs`

## 根拠

- `nepl-core/src/resource/model.rs` には `RawMemoryOp` enum がある一方、同じ raw memory operation を `InternalEffect::UnsafeMemory` と `EffectOp::UnsafeMemory` では `String` として保持していた。
- `ResourceEffectBoundaryDiagnostic::UnsafeMemoryInPureFunction` も operation を `String` として保持しており、Resource IR effect boundary の分岐で raw operation の enum 網羅性が効かなかった。
- `RawMemoryOp::Other { name: String }` は新しい raw operation を enum 追加なしに通せるため、静的検査の意図と逆向きの escape hatch になっていた。

## 問題

Resource IR has RawMemoryOp enum, but InternalEffect::UnsafeMemory and EffectOp::UnsafeMemory keep the raw memory operation as a String. This weakens exhaustiveness and lets new raw memory operations bypass compile-time match coverage.

## 影響

Static check correctness depends on raw memory operations being exhaustively classified. String operations make Resource IR effect boundary diagnostics harder to maintain and conflict with the enum-based diagnostic/static-check policy.

## 修正方針

Move raw memory operation identity into an enum shared by internal effects and Resource IR. Remove the catch-all raw memory operation variant from Resource IR lowering so new known operations must be explicitly mapped.

## 対応

- `RawMemoryOp` を compiler-wide な raw memory operation enum として `effects` 側へ移し、Resource IR model から再 export する形へ整理した。
- `InternalEffect::{InternalAlloc, UnsafeMemory}` と `EffectOp::UnsafeMemory`、`ResourceEffectBoundaryDiagnostic::UnsafeMemoryInPureFunction` が `String` ではなく `RawMemoryOp` を保持するようにした。
- `RawMemoryOp::Other` を削除し、raw helper / intrinsic marker は既知 enum variant へ明示的に mapping されない限り raw operation として扱われないようにした。
- すべての raw memory marker が `RawMemoryOp` へ mapping されることを `effects` test で固定した。
- 親 issue [ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04](./ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04.md) の Stage 5 effect model 進捗として扱う。

## 検証

- `cargo test -p nepl-core --test effects -- --nocapture`: 23 passed
- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 155 passed
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `node nodesrc/issues.js check`: passed
