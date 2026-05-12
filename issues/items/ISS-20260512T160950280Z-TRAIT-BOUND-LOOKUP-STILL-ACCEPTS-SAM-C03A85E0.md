---
id: ISS-20260512T160950280Z-TRAIT-BOUND-LOOKUP-STILL-ACCEPTS-SAM-C03A85E0
title: "Trait bound lookup still accepts same-label TypeId fallback"
area: core
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-12
target: "nepl-core/src/typecheck/traits.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md"
---

# ISS-20260512T160950280Z-TRAIT-BOUND-LOOKUP-STILL-ACCEPTS-SAM-C03A85E0: Trait bound lookup still accepts same-label TypeId fallback

## 概要

type_param_has_trait_application_bound still treats unrelated TypeId variables with the same label as the same bounded type parameter. After duplicate lookup paths were removed, this label fallback remains the last authority that can mix same-label type parameters from different scopes.

## 対象

- `nepl-core/src/typecheck/traits.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md`

## 根拠

- `doc/neplg2/abstraction_static_verification_plan.md` Stage 3 は label fallback を削除し、必要な対応を明示的な substitution map へ寄せることを求めている。
- `type_param_has_trait_application_bound` は exact `TypeId` / resolved `TypeId` lookup が失敗した後、`TypeKind::Var` の label 文字列が一致する別 `TypeId` の bounds を成功扱いしていた。
- 直前の `ISS-20260512T160137255Z-TRAIT-BOUND-LOOKUP-DUPLICATES-TYPEID-1694BE55` で lookup authority は一箇所に集約済みなので、この helper の fallback が残る最後の同名混線経路になっていた。

## 問題

type_param_has_trait_application_bound still treats unrelated TypeId variables with the same label as the same bounded type parameter. After duplicate lookup paths were removed, this label fallback remains the last authority that can mix same-label type parameters from different scopes.

## 影響

Static verification of generic trait bounds can succeed by label text rather than declaration identity. That conflicts with the Stage 3 BoundEnv/TypeParamId direction and makes same-label/different-scope safety unprovable.

## 修正方針

Remove the same-label fallback from the typed helper, rely on exact resolved TypeId identity and explicit substitution mapping, and extend source policy so trait bound lookup cannot reintroduce label-based matching.

## 対応記録

- `type_param_has_trait_application_bound` から `TypeKind::Var` label を比較する fallback を削除した。
- lookup は直接 key と resolved `TypeId` の明示照合だけにした。
- source policy に typed helper 内で `same_label` と `v.label.as_deref` を使わない検査を追加した。

## 検証

- `cargo check -p nepl-core --tests`: pass
- `cargo test -p nepl-core generics -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 trait_bound_satisfied_in_generic -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 generic_trait_impl_method_resolves_by_trait_args -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 trait_method_call_with_impl_compiles -- --nocapture`: pass
- `node nodesrc/test_abstraction_static_verification_policy.js`: pass
- `node nodesrc/issues.js check --dir issues`: pass
