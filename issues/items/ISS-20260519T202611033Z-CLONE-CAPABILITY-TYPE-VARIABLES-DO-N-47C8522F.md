---
id: ISS-20260519T202611033Z-CLONE-CAPABILITY-TYPE-VARIABLES-DO-N-47C8522F
title: "Clone-capability type variables do not constrain impl target matching"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-19
updated: 2026-05-20
target: "nepl-core/src/types.rs; nepl-core/src/typecheck/driver.rs; nepl-core/tests/neplg2.rs"
---

# ISS-20260519T202611033Z-CLONE-CAPABILITY-TYPE-VARIABLES-DO-N-47C8522F: Clone-capability type variables do not constrain impl target matching

## 概要

TypeCtx records TypeVar.clone_cap from trait bounds, but type_pattern_matches ignores clone_cap and only checks copy/drop capabilities. A generic impl such as impl<.T: Clone> Marker for Wrap<.T> can therefore match Wrap<NonClone> during trait resolution.

## 対象

- `nepl-core/src/types.rs; nepl-core/src/typecheck/driver.rs; nepl-core/tests/neplg2.rs`

## 根拠

- `collect_type_params` は trait capability bound を `TypeVar.copy_cap` / `clone_cap` / `drop_cap` へ記録する。
- `TypeCtx::pattern_var_capabilities_match` は従来 `copy_cap` と `drop_cap` だけを検査し、`clone_cap` を無視していた。
- `TypeCtx` は Copy / Drop impl target だけを registry に持ち、Clone impl target を後続の type-pattern matching から参照できなかった。

## 問題

TypeCtx records TypeVar.clone_cap from trait bounds, but type_pattern_matches ignores clone_cap and only checks copy/drop capabilities. A generic impl such as impl<.T: Clone> Marker for Wrap<.T> can therefore match Wrap<NonClone> during trait resolution.

## 影響

Trait impl selection can apply an implementation outside its declared bound, weakening abstraction and static verification. This lets code rely on a Clone-protected implementation for a type whose source does not prove Clone capability.

## 修正方針

Track clone-capability impl targets in TypeCtx, add a has_clone query, enforce clone_cap in type-pattern variable matching, and register clone impl targets alongside copy/drop impl targets. Preserve Copy-implies-Clone semantics for type variables because Copy impls are already required to have a clone-capability impl.

## 検証

Add Rust integration tests that reject Clone-bounded generic impls for non-Clone payloads and accept them once a Clone impl exists; run focused trait/type tests and issue validation.

## 対応結果

- `TypeCtx` に `clone_impl_targets` を追加し、snapshot / rollback / clone にも含めた。
- `register_clone_impl_target` / `has_clone_impl_target` / `has_clone` を追加し、recursive blanket impl が自分自身を proof として使わないように query stack を持たせた。
- `pattern_var_capabilities_match` が `clone_cap` を必ず検査するようにした。
- typecheck driver が clone capability impl target を copy/drop と同じ impl table registration phase で登録するようにした。
- `Copy` は `Clone` を要求する既存設計なので、type variable capability では `copy_cap` を `clone_cap` としても扱う。

## 回帰テスト

- `clone_capability_bound_constrains_generic_impl_target`
- `clone_capability_bound_allows_matching_clone_payload`
- `recursive_clone_capability_impl_does_not_prove_itself`

## 検証結果

- `cargo test -p nepl-core --test neplg2 clone_capability_bound -- --nocapture`: passed
- `cargo test -p nepl-core --test neplg2 recursive_clone_capability_impl_does_not_prove_itself -- --nocapture`: passed
- `cargo test -p nepl-core --test neplg2 trait_ -- --nocapture`: passed
- `cargo test -p nepl-core --test neplg2 copy_impl -- --nocapture`: passed
