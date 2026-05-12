---
id: ISS-20260512T172241782Z-TRAIT-TYPE-PARAMETER-BOUNDS-STILL-EX-09CE8755
title: "Trait type parameter bounds still expose raw BTreeMap state"
area: core
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-13
target: "nepl-core/src/typecheck/traits.rs; nepl-core/src/typecheck/context.rs; nepl-core/src/typecheck/env.rs; nepl-core/src/typecheck/function_check.rs; nepl-core/src/typecheck/trait_bound_apply.rs; nepl-core/src/typecheck/trait_check.rs; nepl-core/src/typecheck/selected_call_apply.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md"
---

# ISS-20260512T172241782Z-TRAIT-TYPE-PARAMETER-BOUNDS-STILL-EX-09CE8755: Trait type parameter bounds still expose raw BTreeMap state

## 概要

Stage 3 still passes type parameter trait bounds as raw BTreeMap<TypeId, Vec<TraitBound>> through BlockChecker, BindingKind, function_check, selected_call_apply, and trait_bound_apply. Lookup authority is centralized functionally, but the raw map representation remains public inside typecheck and can be iterated or cloned without going through the typed lookup boundary.

## 対象

- `nepl-core/src/typecheck/traits.rs; nepl-core/src/typecheck/context.rs; nepl-core/src/typecheck/env.rs; nepl-core/src/typecheck/function_check.rs; nepl-core/src/typecheck/trait_bound_apply.rs; nepl-core/src/typecheck/trait_check.rs; nepl-core/src/typecheck/selected_call_apply.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md`

## 根拠

- `BlockChecker` は `type_param_bounds: BTreeMap<TypeId, Vec<TraitBound>>` を field として持っていた。
- `BindingKind::Func` と `check_function` は同じ raw map 型を引き回していた。
- `trait_bound_apply.rs` と `selected_call_apply.rs` は raw map を clone / iterate できる状態で、Stage 3 の BoundEnv 導入方針に反していた。
- 既に label fallback は削除済みだが、raw map が露出している限り将来の TypeParamId / BoundEnv authority を bypass できる。

## 問題

Stage 3 still passes type parameter trait bounds as raw BTreeMap<TypeId, Vec<TraitBound>> through BlockChecker, BindingKind, function_check, selected_call_apply, and trait_bound_apply. Lookup authority is centralized functionally, but the raw map representation remains public inside typecheck and can be iterated or cloned without going through the typed lookup boundary.

## 影響

Future TypeParamId / BoundEnv work can be bypassed accidentally by direct raw-map access. That weakens static verification of generic trait bounds and makes same-label/different-scope regressions harder to prevent with source policy.

## 修正方針

Introduce BoundEnv as the only typecheck-side container for type parameter trait bounds. Move insert, empty, iteration, resolved lookup, and trait-application satisfaction into BoundEnv methods, then update call sites and policy to reject raw BTreeMap<TypeId, Vec<TraitBound>> fields/parameters.

## 対応記録

- `BoundEnv` を `typecheck/traits.rs` に追加し、type parameter trait bounds の唯一の container にした。
- `BoundEnv::new` / `is_empty` / `insert` / `iter` / `has_trait_application_bound` を追加し、resolved `TypeId` lookup と trait application matching を BoundEnv に閉じ込めた。
- `BlockChecker`、`BindingKind::Func`、`check_function`、nested function check、selected callable application、trait bound application を `BoundEnv` 経由に移行した。
- `type_param_has_trait_application_bound` は raw map を受け取らず、`BoundEnv` の lookup authority へ委譲する形にした。
- `nodesrc/test_abstraction_static_verification_policy.js` に `BoundEnv` 必須化と raw `BTreeMap<TypeId, Vec<TraitBound>>` 境界再導入禁止を追加した。
- 後続の `ISS-20260512T173702516Z-BOUNDENV-STILL-KEYS-TYPE-PARAMETER-B-792D9BA4` で、`BoundEnv` 内部 key も `TypeParamId` newtype へ移行し、raw `TypeId` key は source policy で禁止した。

## 検証

cargo check -p nepl-core --tests; cargo test -p nepl-core --test neplg2 trait_bound_satisfied_in_generic -- --nocapture; cargo test -p nepl-core --test neplg2 trait_bound_missing_impl_is_error -- --nocapture; node nodesrc/test_abstraction_static_verification_policy.js

## 確認済み

- `cargo check -p nepl-core --tests`: pass
- `node nodesrc/test_abstraction_static_verification_policy.js`: pass
- `cargo test -p nepl-core --test neplg2 trait_bound_satisfied_in_generic -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 trait_bound_missing_impl_is_error -- --nocapture`: pass
