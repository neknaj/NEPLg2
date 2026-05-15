---
id: ISS-20260515T024851827Z-RAW-MEMORY-OPERATIONS-SHARE-A-FILE-W-2A06192D
title: "Raw memory operations share a file-wide boundary capability"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-15
updated: 2026-05-15
target: "nepl-core/src/source_map.rs, nepl-core/src/source_capability/raw_memory.rs, nepl-core/src/loader.rs, nepl-core/src/typecheck/effect_check.rs, nepl-core/src/compiler.rs, nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260515T024851827Z-RAW-MEMORY-OPERATIONS-SHARE-A-FILE-W-2A06192D: Raw memory operations share a file-wide boundary capability

## 概要

actual raw memory operations and raw body memory instructions were collapsed into a file-wide RawMemoryBoundary capability, so evidence for one raw operation could authorize unrelated raw operations in the same compiler-owned source file.

## 対象

- `nepl-core/src/source_map.rs, nepl-core/src/source_capability/raw_memory.rs, nepl-core/src/loader.rs, nepl-core/src/typecheck/effect_check.rs, nepl-core/src/compiler.rs, nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- `SourceCapability` は raw memory source proof を `RawMemoryBoundary` の bool として保持していた。
- `loader` は raw helper call / raw body instruction / raw address helper / restricted constructor を同じ capability に畳んでいた。
- `typecheck/effect_check.rs` と `compiler.rs` の raw boundary suppression は、診断中の `RawMemoryOp` や raw body backend operation と capability を照合していなかった。

## 問題

actual raw memory operations and raw body memory instructions were collapsed into a file-wide RawMemoryBoundary capability, so evidence for one raw operation could authorize unrelated raw operations in the same compiler-owned source file.

## 影響

Stage 6 source proof cannot state which raw operation was proven, weakening memory/effect safety and making raw authority broader than the parsed source evidence.

## 修正方針

Represent raw memory and raw body operations as enum-valued source capabilities, collect exact operation evidence from parsed source/raw bodies, and check the operation being used at typecheck/resource diagnostic filtering time.

## 解決内容

- `SourceCapability::RawMemoryStructuralBoundary`、`RawMemoryOperationBoundary(RawMemoryOp)`、`RawBodyMemoryOperationBoundary(RawBodyMemoryOp)` に分離した。
- source capability scanner は raw address identity helper / restricted compiler-memory constructor を structural evidence とし、actual raw helper / raw intrinsic は `RawMemoryOp`、`#wasm` / `#llvm` memory instruction は `RawBodyMemoryOp` として収集する。
- checked owner wrapper は raw structural evidence ではなく、実際に内部で使う raw operation だけが operation evidence になる。
- typecheck の raw intrinsic / raw body gate と ResourceEffectBoundary diagnostic suppression は、診断対象の operation と file capability を照合する。
- `ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc` は raw identity の発生元 `RawMemoryOp` を保持し、checked wrapper 名を raw operation として whitelist せず、`Alloc` / `Realloc` 由来 identity と source capability を照合する。
- bare-source migration regression は実 stdlib と同じ Copy capability fixture を持つ形に直し、`i32` 引数の読み出しを non-Copy move と誤診断しない条件で raw intrinsic を検証する。
- source policy に operation-specific capability と compiler diagnostic suppression の監査を追加した。

## 検証

- `cargo fmt -p nepl-core --check`
- `cargo test -p nepl-core raw_memory_boundary --lib -- --nocapture`
- `cargo test -p nepl-core source_capabilities --lib -- --nocapture`
- `cargo test -p nepl-core source_map_keeps_capabilities_per_file --lib -- --nocapture`
- `cargo test -p nepl-core raw_body_memory --test effects -- --nocapture`
- `cargo test -p nepl-core raw_memory --test effects -- --nocapture`
- `cargo test -p nepl-core resource_ir_effect_check --test resource_ir -- --nocapture`
- `cargo test -p nepl-core resource_effect_gate --lib -- --nocapture`
- `node nodesrc/test_static_check_boundary_responsibility.js`
- `trunk build`
- `node nodesrc/tests.js -i stdlib/core/mem.nepl -i stdlib/core/mem/types.nepl -i stdlib/core/mem/internal.nepl -i stdlib/core/mem/pointer/alloc.nepl -i stdlib/core/mem/pointer/region.nepl -i stdlib/core/mem/pointer/scalar.nepl -i stdlib/alloc/collections/vec/storage/api.nepl -i stdlib/alloc/collections/vec/storage/view.nepl -i stdlib/alloc/collections/vec/storage/cleanup.nepl --no-tree -o tmp/agent1-raw-operation-specific-capability-doctests-after-identity-origin.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/issues.js check --dir issues`
