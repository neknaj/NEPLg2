---
id: ISS-20260512T163228542Z-PENDING-TRAIT-CHECKS-STILL-USE-POSIT-FB7F1082
title: "Pending trait checks still use positional tuple state"
area: core
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-12
target: "nepl-core/src/typecheck/traits.rs; nepl-core/src/typecheck/context.rs; nepl-core/src/typecheck/function_check.rs; nepl-core/src/typecheck/trait_bound_apply.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md"
---

# ISS-20260512T163228542Z-PENDING-TRAIT-CHECKS-STILL-USE-POSIT-FB7F1082: Pending trait checks still use positional tuple state

## 概要

pending_trait_bound_checks is stored as Vec<(TraitBound, TypeId, Span)> and later destructured as (bound, ty, span). The meaning of the TypeId and Span fields is positional, not encoded in the type system.

## 対象

- `nepl-core/src/typecheck/traits.rs; nepl-core/src/typecheck/context.rs; nepl-core/src/typecheck/function_check.rs; nepl-core/src/typecheck/trait_bound_apply.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md`

## 根拠

- `doc/neplg2/abstraction_static_verification_plan.md` は `pending_trait_bound_checks: Vec<(TraitBound, TypeId, Span)>` を、field の意味が型名だけでは読み取りにくい残件として挙げている。
- `BlockChecker` は pending trait check を tuple で保持し、`trait_bound_apply.rs` は `(substituted_bound, inferred_arg, span)` を enqueue し、`function_check.rs` は `(bound, ty, span)` として destructure していた。
- `TypeId` が「未解決なら後で検査する対象型」であること、`Span` が「診断位置」であることが型に表れていなかった。

## 問題

pending_trait_bound_checks is stored as Vec<(TraitBound, TypeId, Span)> and later destructured as (bound, ty, span). The meaning of the TypeId and Span fields is positional, not encoded in the type system.

## 影響

Future static-check changes can swap or misuse pending target type and diagnostic span without compiler help. This conflicts with the enum/structured data policy for type-safety critical checks.

## 修正方針

Introduce a PendingTraitCheck struct with named bound, target_ty, and span fields; use it in BlockChecker, trait_bound_apply, and function_check; add source policy to reject the tuple model.

## 対応記録

- `PendingTraitCheck { bound, target_ty, span }` を追加した。
- `BlockChecker::pending_trait_bound_checks` を `Vec<PendingTraitCheck>` に変更した。
- `trait_bound_apply.rs` は named struct literal で pending check を enqueue するようにした。
- `function_check.rs` は `PendingTraitCheck` の named field を destructure して検査するようにした。
- source policy に positional tuple model と tuple push 再導入禁止を追加した。

## 検証

- `cargo check -p nepl-core --tests`: pass
- `cargo test -p nepl-core --test neplg2 trait_bound_satisfied_in_generic -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 trait_bound_missing_impl_is_error -- --nocapture`: pass
- `node nodesrc/test_abstraction_static_verification_policy.js`: pass
- `node nodesrc/issues.js check --dir issues`: pass
