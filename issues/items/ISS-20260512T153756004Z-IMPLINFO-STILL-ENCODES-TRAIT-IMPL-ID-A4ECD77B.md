---
id: ISS-20260512T153756004Z-IMPLINFO-STILL-ENCODES-TRAIT-IMPL-ID-A4ECD77B
title: "ImplInfo still encodes trait impl identity with optional fields"
area: core
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-12
target: "nepl-core/src/typecheck/traits.rs; nepl-core/src/typecheck/driver.rs; nepl-core/src/typecheck/function_check.rs; nepl-core/src/typecheck/trait_check.rs; nepl-core/src/typecheck/trait_call_apply.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md"
---

# ISS-20260512T153756004Z-IMPLINFO-STILL-ENCODES-TRAIT-IMPL-ID-A4ECD77B: ImplInfo still encodes trait impl identity with optional fields

## 概要

ImplInfo stores trait implementation identity as trait_name, trait_base_name, trait_args, and trait_self_ty optional/split fields. This makes inherent-vs-trait impl state implicit and lets call sites inspect optional strings instead of forcing exhaustive ImplKind matching.

## 対象

- `nepl-core/src/typecheck/traits.rs; nepl-core/src/typecheck/driver.rs; nepl-core/src/typecheck/function_check.rs; nepl-core/src/typecheck/trait_check.rs; nepl-core/src/typecheck/trait_call_apply.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md`

## 根拠

- `doc/neplg2/abstraction_static_verification_plan.md` Stage 2 は `ImplInfo` を optional field combination ではなく `ImplKind` enum へ移行することを求めている。
- 変更前の `ImplInfo` は `trait_name: Option<String>`、`trait_base_name: Option<String>`、`trait_args: Vec<TypeId>`、`trait_self_ty: Option<TypeId>` を同時に持っていた。
- `function_check.rs` / `trait_check.rs` / `trait_call_apply.rs` は `imp.trait_base_name` と `imp.trait_args` を直接読み、typecheck-side impl identity の分岐が enum match で強制されていなかった。

## 問題

ImplInfo stores trait implementation identity as trait_name, trait_base_name, trait_args, and trait_self_ty optional/split fields. This makes inherent-vs-trait impl state implicit and lets call sites inspect optional strings instead of forcing exhaustive ImplKind matching.

## 影響

Trait impl matching can silently ignore malformed combinations of optional fields, and future abstraction work can reintroduce string-rendered authority without Rust match exhaustiveness. This conflicts with the enum-based static verification policy for type safety.

## 修正方針

Introduce ImplKind with explicit Inherent and Trait variants, store TraitApplication and self TypeId in the Trait variant, and route typecheck impl matching through typed helper methods rather than optional string fields.

## 対応記録

- `ImplKind` を追加し、`ImplInfo` を `kind: ImplKind` と `target_ty` に再構成した。
- trait impl identity は `ImplKind::Trait { application: TraitApplication, self_ty: TypeId }` で保持し、表示名や optional field の組み合わせから分離した。
- duplicate impl、deferred trait check、trait bound satisfaction、trait method application は `ImplInfo::matches_trait_application` / `matches_same_trait_impl` 経由の typed matching にした。
- `nodesrc/test_abstraction_static_verification_policy.js` は `ImplInfo` の optional string field 再導入と `imp.trait_base_name` / `imp.trait_args` 直読み再導入を拒否する。

## 検証

- `cargo check -p nepl-core --tests`: pass
- `cargo test -p nepl-core generics -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 generic_trait_impl_method_resolves_by_trait_args -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 impl_type_params_in_trait_args_allowed_for_concrete_target -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 trait_bound_satisfied_in_generic -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 impl_trait_type_arg_count_has_type_code -- --nocapture`: pass
- `node nodesrc/test_abstraction_static_verification_policy.js`: pass
- `node nodesrc/issues.js check --dir issues`: pass
- `cargo test -p nepl-core trait -- --nocapture`: fail。`generic_store_after_generic_trait_probe_preserves_struct` が `origin/main` でも同じ `Effect(PureCallsImpure)` で失敗するため、`ISS-20260512T155153362Z-GENERIC-TRAIT-PROBE-REGRESSION-FIXTU-65BB07DB` として別 issue 化した。
