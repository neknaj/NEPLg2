---
id: ISS-20260517T022637369Z-RAW-IDENTITY-ESCAPE-SUPPRESSION-USES-0DD1EDAA
title: "raw identity escape suppression uses return-span aggregate source proof"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/resource/effect_identity.rs, nepl-core/src/resource/effect_raw_memory_identity.rs, nepl-core/src/compiler.rs, nepl-core/src/source_map.rs"
---

# ISS-20260517T022637369Z-RAW-IDENTITY-ESCAPE-SUPPRESSION-USES-0DD1EDAA: raw identity escape suppression uses return-span aggregate source proof

## 概要

RawAddressEscapeFromInternalAlloc diagnostics only carry the return terminator span, while RawIdentityTable records only RawMemoryOp without the source span that created the raw identity. compiler.rs then accepts any matching source capability within the broad return span. A compiler-owned source can therefore include an unrelated proven alloc/realloc use site in the same return expression and accidentally suppress an escape for a different raw identity.

## 対象

- `nepl-core/src/resource/effect_identity.rs`

## 根拠

- `RawIdentityTable` が `RawMemoryOp` だけを保持しており、raw identity を作った exact source span を保持していなかった。
- `RawAddressEscapeFromInternalAlloc` diagnostic は return terminator span だけを持っていた。
- `compiler.rs` は `raw_memory_operation_boundary_allowed_within(return_span, op)` で診断を抑制していたため、return expression 内の unrelated raw operation proof が別 identity の escape まで許可し得た。

## 問題

RawAddressEscapeFromInternalAlloc diagnostics only carry the return terminator span, while RawIdentityTable records only RawMemoryOp without the source span that created the raw identity. compiler.rs then accepts any matching source capability within the broad return span. A compiler-owned source can therefore include an unrelated proven alloc/realloc use site in the same return expression and accidentally suppress an escape for a different raw identity.

## 影響

This weakens Stage 6 source proof exactness and violates the generic proof design: the proof artifact is not attached to the resource fact being checked, so static-check bugs become harder to catch and raw identity escape suppression can be overbroad.

## 修正方針

Carry RawIdentityOrigin { operation, span } through RawIdentityTable, RawMemoryIdentityTable, function return summaries, and ResourceEffectBoundaryDiagnostic. Make compiler gating require exact source capability at the origin span, not any capability within the return span.

## 検証

Add focused regression tests for exact origin-span gating and run nepl-core focused resource/effect/source-policy checks.

## 2026-05-17 Agent 1 修正

`RawIdentityTable` と `RawMemoryIdentityTable` の保持内容を `RawMemoryOp` の集合から `RawIdentityOrigin { operation, span }` の集合へ変更した。raw memory operation の span を origin として記録し、copy / move / borrow / aggregate construction / branch merge / raw memory store-load / function return summary を通して origin を伝播する。

`ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc` は return span に加えて `origin_span` を持つ。`compiler.rs` の suppression gate は return span 内を探索せず、origin span の exact `raw_memory_operation_boundary_allowed_at` または exact `raw_memory_structural_boundary_allowed_at` だけを見る。`SourceMap` / `SourceCapabilities` から broad `allowed_within` query も削除し、source policy で再導入を拒否する。

検証:
- `cargo check -p nepl-core`
- `node nodesrc/test_static_check_boundary_responsibility.js`
- `cargo test -p nepl-core resource_effect_gate_requires_raw_identity_origin_span_capability -- --nocapture`
- `cargo test -p nepl-core resource_ir_effect_check_propagates_internal_alloc_return_summary --test resource_ir -- --nocapture`
- `cargo test -p nepl-core source_map::tests -- --nocapture`
- `cargo test -p nepl-core raw_identity --test resource_ir -- --nocapture`
- `cargo fmt -p nepl-core --check`
