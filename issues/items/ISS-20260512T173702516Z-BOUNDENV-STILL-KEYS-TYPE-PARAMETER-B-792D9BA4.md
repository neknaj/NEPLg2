---
id: ISS-20260512T173702516Z-BOUNDENV-STILL-KEYS-TYPE-PARAMETER-B-792D9BA4
title: "BoundEnv still keys type parameter bounds by raw TypeId"
area: core
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-13
target: "nepl-core/src/typecheck/traits.rs; nepl-core/src/typecheck/block_check.rs; nepl-core/src/typecheck/driver.rs; nepl-core/src/typecheck/trait_bound_apply.rs; nepl-core/src/typecheck/trait_check.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md"
---

# ISS-20260512T173702516Z-BOUNDENV-STILL-KEYS-TYPE-PARAMETER-B-792D9BA4: BoundEnv still keys type parameter bounds by raw TypeId

## 概要

BoundEnv removes raw BTreeMap<TypeId, Vec<TraitBound>> from most typecheck boundaries, but the container itself still stores type parameter bounds under raw TypeId keys. Call sites can still insert arbitrary TypeId values, so type parameter declaration identity is protected by convention rather than by Rust's type system.

## 対象

- `nepl-core/src/typecheck/traits.rs`

## 根拠

- `BoundEnv` の field が `BTreeMap<TypeId, Vec<TraitBound>>` のままだった。
- `collect_type_params`、nested function check、top-level driver が `BoundEnv::insert` へ raw `TypeId` を渡していた。
- `trait_bound_apply.rs` / `trait_check.rs` は `BoundEnv::iter` から raw `TypeId` key を受け取り、宣言 ID と通常の type ID を型で区別できなかった。
- 詳細計画: [NEPLg2 abstraction static verification plan Stage 3](../../doc/neplg2/abstraction_static_verification_plan.md#stage-3-boundenv-%E3%81%A8-type-parameter-identity)

## 問題

BoundEnv removes raw BTreeMap<TypeId, Vec<TraitBound>> from most typecheck boundaries, but the container itself still stores type parameter bounds under raw TypeId keys. Call sites can still insert arbitrary TypeId values, so type parameter declaration identity is protected by convention rather than by Rust's type system.

## 影響

Future generic trait-bound work can accidentally mix ordinary TypeId values with type parameter declaration IDs. That weakens the Stage 3 BoundEnv/TypeParamId model, makes same-label/different-scope safety harder to audit, and leaves static verification policy unable to prove that only type parameter declarations enter the bound environment.

## 修正方針

Introduce a TypeParamId newtype and make BoundEnv store BTreeMap<TypeParamId, Vec<TraitBound>>. Require TypeParamId at BoundEnv insertion/iteration boundaries, update trait-bound application and trait checking to unwrap explicitly, and extend the abstraction static verification policy to reject raw TypeId bound keys.

## 対応記録

- `TypeParamId` newtype を `typecheck/traits.rs` に追加し、type parameter declaration identity を raw `TypeId` と区別した。
- `BoundEnv` の内部 key を `BTreeMap<TypeParamId, Vec<TraitBound>>` に変更し、`insert` は `TypeParamId` だけを受け取る形にした。
- `BoundEnv::iter` は `TypeParamId` を返し、`trait_bound_apply.rs` / `trait_check.rs` は `.type_id()` で明示的に unwrap する形にした。
- `collect_type_params`、nested function check、top-level driver の bound collection は `TypeParamId::new(...)` を通して `BoundEnv` に挿入する。
- `nodesrc/test_abstraction_static_verification_policy.js` に TypeParamId 必須化、raw `BTreeMap<TypeId, Vec<TraitBound>>` key 禁止、raw `insert(*p_id, bounds)` 禁止を追加した。

## 検証

cargo check -p nepl-core --tests; node nodesrc/test_abstraction_static_verification_policy.js; cargo test -p nepl-core --test neplg2 trait_bound_satisfied_in_generic -- --nocapture; cargo test -p nepl-core --test neplg2 trait_bound_missing_impl_is_error -- --nocapture; node nodesrc/issues.js check --dir issues

## 確認済み

- `cargo check -p nepl-core --tests`: pass
- `node nodesrc/test_abstraction_static_verification_policy.js`: pass
- `cargo test -p nepl-core --test neplg2 trait_bound_satisfied_in_generic -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 trait_bound_missing_impl_is_error -- --nocapture`: pass
