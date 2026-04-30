---
id: ISS-20260430T064827021Z-RESOURCE-IR-INDIRECT-CALLS-KEEP-UNKN-E9B29774
title: "Resource IR indirect calls keep unknown effect instead of typed effect summary"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-30
updated: 2026-04-30
target: "nepl-core/src/resource/lower.rs, nepl-core/src/resource/effect_check.rs, nepl-core/src/hir.rs, nepl-core/src/typecheck/indirect_apply.rs"
---

# ISS-20260430T064827021Z-RESOURCE-IR-INDIRECT-CALLS-KEEP-UNKN-E9B29774: Resource IR indirect calls keep unknown effect instead of typed effect summary

## 概要

Resource IR lowering emits EffectOp::Unknown for HirExprKind::CallIndirect. Current pure/impure safety is still protected by typecheck, but the Resource IR effect gate only counts Unknown and does not produce a conservative diagnostic or typed effect summary for indirect calls.

## 対象

- `nepl-core/src/resource/lower.rs, nepl-core/src/resource/effect_check.rs, nepl-core/src/hir.rs, nepl-core/src/typecheck/indirect_apply.rs`

## 根拠

- `nepl-core/src/typecheck/indirect_apply.rs` は indirect call の callee が function value であることは確認するが、HIR の `CallIndirect` には effect field を持たせていない。
- `nepl-core/src/resource/lower.rs` は `HirExprKind::CallIndirect` を lowering するときに `EffectOp::Unknown { reason: "indirect call" }` を生成する。
- `nepl-core/src/resource/effect_check.rs` は `EffectOp::Unknown` を `unknown_ops` として数えるだけで、compiler diagnostic にはしない。
- probe では pure context から impure function value を indirect call すると typecheck が `effect.pure.calls_impure` で拒否したため、現時点の safety は typecheck 防壁に依存している。

## 問題

Resource IR lowering emits EffectOp::Unknown for HirExprKind::CallIndirect. Current pure/impure safety is still protected by typecheck, but the Resource IR effect gate only counts Unknown and does not produce a conservative diagnostic or typed effect summary for indirect calls.

## 影響

Resource IR cannot become the final authority for effect and resource safety while indirect calls lose their function-type effect at lowering. Self-host could copy this gap and treat unknown callback effects as non-errors, weakening static check correctness.

## 修正方針

Carry function value effect information into HirExprKind::CallIndirect or ResourceOp::IndirectCall, lower it as a typed EffectOp/UserCall summary instead of opaque Unknown, and make unknown resource effects either impossible after lowering coverage or a conservative compiler error outside explicitly allowed internal boundaries.

## 対応

- `HirExprKind::CallIndirect` に `effect` を追加し、`typecheck/indirect_apply.rs` で callee の関数型から `Effect` を必ず取得するようにした。
- `EffectOp::IndirectCall { effect }` を追加し、Resource IR lowering では `EffectOp::Unknown { reason: "indirect call" }` を生成しないようにした。
- Resource effect boundary gate に `ImpureCallInPureFunction` を追加し、純粋関数内の impure な直接/間接呼び出しを `effect.pure.calls_impure` へ写像できるようにした。
- raw memory boundary の許可判定が effect の呼び出し診断まで誤って抑制しないよう、raw memory 系診断だけを boundary 対象に分離した。

## 検証

Add Resource IR tests for indirect pure and impure function values, a compile_fail doctest for pure indirect impure calls, and source-policy or coverage checks ensuring EffectOp::Unknown is not used as the normal indirect-call representation.

- `cargo check -p nepl-core --tests`
- `cargo test -p nepl-core --test resource_ir -- --nocapture`
- `cargo test -p nepl-core --test effects pure_indirect_impure_function_value_is_rejected -- --nocapture`
- `cargo test -p nepl-core resource_effect_gate_maps_impure_indirect_call_to_effect_code -- --nocapture`
- `cargo fmt --check -p nepl-core`
- `trunk build`
- `node nodesrc/tests.js -i tests/compiler/indirect_effect.n.md --no-tree -o tmp/indirect-effect-typed-resource-ir.json -j 1 --dist web/dist`
