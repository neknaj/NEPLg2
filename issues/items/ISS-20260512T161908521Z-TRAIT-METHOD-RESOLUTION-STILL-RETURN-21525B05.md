---
id: ISS-20260512T161908521Z-TRAIT-METHOD-RESOLUTION-STILL-RETURN-21525B05
title: "Trait method resolution still returns raw optional FuncRef"
area: core
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-12
target: "nepl-core/src/typecheck/trait_call_apply.rs; nepl-core/src/typecheck/selected_call_apply.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md"
---

# ISS-20260512T161908521Z-TRAIT-METHOD-RESOLUTION-STILL-RETURN-21525B05: Trait method resolution still returns raw optional FuncRef

## 概要

trait_call_apply.rs has separate selected-call and unbound-call paths that infer trait method receiver, trait arguments, and FuncRef::Trait directly. infer_selected_trait_method_callee returns Option<FuncRef>, so NotTraitMethod, missing receiver, unsatisfied bound, and successful resolution are not represented by a typed resolution enum.

## 対象

- `nepl-core/src/typecheck/trait_call_apply.rs; nepl-core/src/typecheck/selected_call_apply.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md`

## 根拠

- `doc/neplg2/abstraction_static_verification_plan.md` Stage 4 は `TraitMethodResolution` enum を追加し、receiver inference、expected self type、type argument inference、effect check を分類することを求めている。
- 変更前の `infer_selected_trait_method_callee` は `Option<FuncRef>` を返しており、trait method ではない場合、self type が推論できない場合、bound 不一致、成功を型で区別していなかった。
- `apply_unbound_trait_method_call` は selected callable 経路とほぼ同じ self/type-argument inference を再実装し、直接 `FuncRef::Trait` を生成していた。

## 問題

trait_call_apply.rs has separate selected-call and unbound-call paths that infer trait method receiver, trait arguments, and FuncRef::Trait directly. infer_selected_trait_method_callee returns Option<FuncRef>, so NotTraitMethod, missing receiver, unsatisfied bound, and successful resolution are not represented by a typed resolution enum.

## 影響

Trait method resolution is hard to audit and future changes can bypass receiver/bound/effect checks or silently fall back to user calls. This conflicts with the static verification policy that branching must be enum-based and exhaustively matched.

## 修正方針

Introduce a TraitMethodResolution enum and a TraitMethodCall model, share receiver/type-argument inference through one resolver, make selected-call and unbound-call callers match the enum explicitly, and keep diagnostics at typed failure variants.

## 対応記録

- `TraitMethodResolution` enum と `TraitMethodCall` model を追加した。
- selected callable 経路は `resolve_selected_trait_method_call` を呼び、`selected_call_apply.rs` 側で enum を明示的に match するようにした。
- unbound trait method 経路は同じ `resolve_trait_method_call` を使い、`MissingSelfType` / `UnsatisfiedBound` / `PureCallsImpure` の typed failure variant から診断を生成するようにした。
- `Option<FuncRef>` を返す旧 `infer_selected_trait_method_callee` は削除した。
- source policy に `TraitMethodResolution` variants、旧 optional helper 禁止、selected callable 側の enum match を追加した。

## 検証

- `cargo check -p nepl-core --tests`: pass
- `cargo test -p nepl-core --test neplg2 trait_method_call_with_impl_compiles -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 pure_function_calling_impure_trait_method_has_effect_code -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 trait_method_not_found_has_type_code -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 trait_method_type_args_unsupported_has_type_code -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 trait_bound_missing_impl_is_error -- --nocapture`: pass
- `node nodesrc/test_abstraction_static_verification_policy.js`: pass
- `node nodesrc/issues.js check --dir issues`: pass
