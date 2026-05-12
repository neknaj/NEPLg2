---
id: ISS-20260512T165852784Z-HIR-TRAIT-CALLS-STILL-SPLIT-TRAIT-AP-405A462B
title: "HIR trait calls still split trait application identity into string fields"
area: core
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-13
target: "nepl-core/src/hir.rs; nepl-core/src/typecheck/trait_call_apply.rs; nepl-core/src/typecheck/driver.rs; nepl-core/src/monomorphize.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md"
---

# ISS-20260512T165852784Z-HIR-TRAIT-CALLS-STILL-SPLIT-TRAIT-AP-405A462B: HIR trait calls still split trait application identity into string fields

## 概要

FuncRef::Trait and HirImpl still store trait_name / trait_base_name / trait_args as separate fields. The monomorphize lookup key is now typed, but the surface HIR boundary can still mix rendered trait names, base trait names, and type argument lists by field convention.

## 対象

- `nepl-core/src/hir.rs; nepl-core/src/typecheck/trait_call_apply.rs; nepl-core/src/typecheck/driver.rs; nepl-core/src/monomorphize.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md`

## 根拠

- `FuncRef::Trait` は `trait_name: String` と `trait_args: Vec<TypeId>` を別 field として持ち、base trait identity と type argument list の対応を型で束ねていなかった。
- `HirImpl` は `trait_name`、`trait_base_name: Option<String>`、`trait_args` を同時に持ち、rendered applied name と base name のどちらを使うべきかが call site convention に依存していた。
- Stage 5 で `monomorphize.rs` 内部 key は `MonoTraitApplication` / `MonoTraitLookupKey` へ移行したが、HIR 境界が split model のままだと再び文字列 authority が流入する。

## 問題

FuncRef::Trait and HirImpl still store trait_name / trait_base_name / trait_args as separate fields. The monomorphize lookup key is now typed, but the surface HIR boundary can still mix rendered trait names, base trait names, and type argument lists by field convention.

## 影響

typecheck and monomorphize can drift at the HIR boundary. A later change can accidentally pass an applied display name where a base trait identity is required, or forget to keep trait_args aligned, weakening static verification for generic trait dispatch and Resource IR lowering.

## 修正方針

Introduce HirTraitApplication and make FuncRef::Trait / HirImpl store that typed application instead of split trait fields. Update lowering/monomorphize/typecheck call sites and source policy to reject the old split HIR model.

## 対応記録

- `HirTraitApplication { base_name, args }` を追加し、display 生成は `display_name(&TypeCtx)` に閉じ込めた。
- `FuncRef::Trait` を `application: HirTraitApplication` / `method` / `self_ty` の model に変更した。
- `HirImpl` を `trait_application: HirTraitApplication` に変更し、`trait_name` / `trait_base_name` / `trait_args` の split fields を削除した。
- typecheck の trait method call 生成、impl HIR 生成、HIR type-id finalization、drop insertion、Resource IR lowering、WASM/LLVM diagnostic、monomorphize lookup を新 model に追従させた。
- source policy に `HirTraitApplication` と `FuncRef::Trait` / `HirImpl` の split field 再導入禁止を追加した。

## 検証

cargo check -p nepl-core --tests; cargo test -p nepl-core --test neplg2 generic_trait_impl_method_resolves_by_trait_args -- --nocapture; cargo test -p nepl-core monomorphize::tests::public_monomorphize_returns_unresolved_trait_calls_without_panicking -- --nocapture; node nodesrc/test_abstraction_static_verification_policy.js

## 確認済み

- `cargo check -p nepl-core --tests`: pass
- `node nodesrc/test_abstraction_static_verification_policy.js`: pass
- `cargo test -p nepl-core --test neplg2 generic_trait_impl_method_resolves_by_trait_args -- --nocapture`: pass
- `cargo test -p nepl-core monomorphize::tests::public_monomorphize_returns_unresolved_trait_calls_without_panicking -- --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_drop_insertion_consumes_checked_scope_and_assignment_points -- --nocapture`: pass
