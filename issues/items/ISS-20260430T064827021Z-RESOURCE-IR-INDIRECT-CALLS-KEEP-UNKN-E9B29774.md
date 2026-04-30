---
id: ISS-20260430T064827021Z-RESOURCE-IR-INDIRECT-CALLS-KEEP-UNKN-E9B29774
title: "Resource IR indirect calls keep unknown effect instead of typed effect summary"
area: core
status: open
resolved: false
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

## 検証

Add Resource IR tests for indirect pure and impure function values, a compile_fail doctest for pure indirect impure calls, and source-policy or coverage checks ensuring EffectOp::Unknown is not used as the normal indirect-call representation.
