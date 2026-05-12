---
id: ISS-20260512T160137255Z-TRAIT-BOUND-LOOKUP-DUPLICATES-TYPEID-1694BE55
title: "Trait bound lookup duplicates TypeId label fallback outside the typed helper"
area: core
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-12
target: "nepl-core/src/typecheck/traits.rs; nepl-core/src/typecheck/trait_check.rs; nepl-core/src/typecheck/trait_call_apply.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md"
---

# ISS-20260512T160137255Z-TRAIT-BOUND-LOOKUP-DUPLICATES-TYPEID-1694BE55: Trait bound lookup duplicates TypeId label fallback outside the typed helper

## 概要

BlockChecker::type_param_has_bound_ref reimplements the same type parameter bound lookup logic that already exists in type_param_has_trait_application_bound, including direct TypeId resolution and label fallback. This leaves two authorities for trait bound satisfaction.

## 対象

- `nepl-core/src/typecheck/traits.rs; nepl-core/src/typecheck/trait_check.rs; nepl-core/src/typecheck/trait_call_apply.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md`

## 根拠

- `doc/neplg2/abstraction_static_verification_plan.md` Stage 3 は `BoundEnv` と type parameter identity を一箇所で管理する設計へ進めることを求めている。
- `nepl-core/src/typecheck/traits.rs` には typed helper `type_param_has_trait_application_bound` がある一方、`trait_check.rs` の `BlockChecker::type_param_has_bound_ref` が同じ lookup と label fallback を再実装していた。
- `trait_call_apply.rs` は `type_param_has_bound_ref` を呼んでおり、Stage 1/2 で削除した `TraitBoundRef` 名に近い古い境界名が残っていた。

## 問題

BlockChecker::type_param_has_bound_ref reimplements the same type parameter bound lookup logic that already exists in type_param_has_trait_application_bound, including direct TypeId resolution and label fallback. This leaves two authorities for trait bound satisfaction.

## 影響

Future BoundEnv/TypeParamId work can update one lookup path while the other silently keeps stale label-based behavior. That weakens the enum/typed static verification policy and makes same-label/different-scope fixes harder to audit.

## 修正方針

Remove the duplicate lookup implementation from trait_check.rs, rename the BlockChecker-facing method to type_param_has_trait_application_bound, delegate to the typed helper in traits.rs, and extend source policy to reject the old type_param_has_bound_ref path.

## 対応記録

- `BlockChecker::type_param_has_bound_ref` を廃止し、`BlockChecker::type_param_has_trait_application_bound` に改名した。
- `trait_check.rs` 側の duplicate label fallback 実装を削除し、`traits.rs` の `type_param_has_trait_application_bound` だけを lookup authority にした。
- `trait_call_apply.rs` の call sites を新しい typed method 名へ更新した。
- source policy に旧 method 名と `trait_check.rs` 側の `same_label` fallback 再導入禁止を追加した。

## 検証

- `cargo check -p nepl-core --tests`: pass
- `cargo test -p nepl-core --test neplg2 trait_bound_satisfied_in_generic -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 generic_trait_impl_method_resolves_by_trait_args -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 trait_method_call_with_impl_compiles -- --nocapture`: pass
- `node nodesrc/test_abstraction_static_verification_policy.js`: pass
- `node nodesrc/issues.js check --dir issues`: pass
