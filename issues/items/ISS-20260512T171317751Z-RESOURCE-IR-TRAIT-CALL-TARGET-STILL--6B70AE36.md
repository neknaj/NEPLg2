---
id: ISS-20260512T171317751Z-RESOURCE-IR-TRAIT-CALL-TARGET-STILL--6B70AE36
title: "Resource IR trait call target still splits trait application identity"
area: core
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-13
target: "nepl-core/src/resource/model.rs; nepl-core/src/resource/lower.rs; nepl-core/src/resource/dump.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md"
---

# ISS-20260512T171317751Z-RESOURCE-IR-TRAIT-CALL-TARGET-STILL--6B70AE36: Resource IR trait call target still splits trait application identity

## 概要

ResourceCallTarget::Trait still stores trait_name and trait_args as separate fields after HIR and monomorphize were moved to typed trait application models.

## 対象

- `nepl-core/src/resource/model.rs; nepl-core/src/resource/lower.rs; nepl-core/src/resource/dump.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md`

## 根拠

- `ResourceCallTarget::Trait` は `trait_name: String` と `trait_args: Vec<TypeId>` を分割 field として持っていた。
- `lower_call_target` は HIR 側の typed `HirTraitApplication` を受け取った後、Resource IR 側で再び split fields に戻していた。
- `resource/dump.rs` は split fields を直接表示しており、Resource IR dump / report が HIR / monomorphize の typed identity model とずれる余地があった。

## 問題

ResourceCallTarget::Trait still stores trait_name and trait_args as separate fields after HIR and monomorphize were moved to typed trait application models.

## 影響

Resource IR dump and static-check audit output can drift from the compiler trait identity model. If a later pass treats the split fields as authority, generic trait call diagnostics and resource reports can lose the same static guarantees restored in HIR and monomorphize.

## 修正方針

Introduce ResourceTraitApplication and make ResourceCallTarget::Trait store application / method / self_ty. Update lowering and dump code, then add source policy coverage rejecting split ResourceCallTarget trait fields.

## 対応記録

- `ResourceTraitApplication { base_name, args }` を追加した。
- `ResourceCallTarget::Trait` を `application: ResourceTraitApplication` / `method` / `self_ty` の model に変更した。
- `resource/lower.rs` は HIR の `HirTraitApplication` から `ResourceTraitApplication` へ明示変換するようにした。
- `resource/dump.rs` は `ResourceTraitApplication` から trait dump 表示を生成するようにした。
- `nodesrc/test_abstraction_static_verification_policy.js` に `ResourceCallTarget::Trait` の split field 再導入禁止を追加した。

## 検証

cargo check -p nepl-core --tests; cargo test -p nepl-core --test resource_ir resource_drop_insertion_consumes_checked_scope_and_assignment_points -- --nocapture; node nodesrc/test_abstraction_static_verification_policy.js

## 確認済み

- `cargo check -p nepl-core --tests`: pass
- `node nodesrc/test_abstraction_static_verification_policy.js`: pass
- `cargo test -p nepl-core --test resource_ir resource_drop_insertion_consumes_checked_scope_and_assignment_points -- --nocapture`: pass
