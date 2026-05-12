---
id: ISS-20260512T193917855Z-TRAIT-METHOD-RESOLUTION-STILL-CARRIE-0BFEEFA9
title: "Trait method resolution still carries raw trait application payload"
area: core
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-13
target: "nepl-core/src/typecheck/trait_call_apply.rs; nepl-core/src/typecheck/trait_check.rs; nepl-core/src/typecheck/traits.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md"
---

# ISS-20260512T193917855Z-TRAIT-METHOD-RESOLUTION-STILL-CARRIE-0BFEEFA9: Trait method resolution still carries raw trait application payload

## 概要

TraitMethodCall keeps trait_name and trait_args as split payload, and UnsatisfiedBound carries applied_trait_name as a rendered string. Trait method resolution is enum-based, but the successful and failure payloads can still mix compiler identity with display text.

## 対象

- `nepl-core/src/typecheck/trait_call_apply.rs; nepl-core/src/typecheck/trait_check.rs; nepl-core/src/typecheck/traits.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md`

## 根拠

- `doc/neplg2/abstraction_static_verification_plan.md` Stage 4/5 は trait method resolution result と trait application identity を typed value として扱い、表示名を static-check authority にしない方針である。
- `TraitMethodResolution` 自体は enum 化済みだったが、`TraitMethodCall` は `trait_name` / `trait_args` の split payload を持ち、`UnsatisfiedBound` は `applied_trait_name` の表示文字列を保持していた。

## 問題

TraitMethodCall keeps trait_name and trait_args as split payload, and UnsatisfiedBound carries applied_trait_name as a rendered string. Trait method resolution is enum-based, but the successful and failure payloads can still mix compiler identity with display text.

## 影響

Future trait resolution changes can bypass typed TraitApplication agreement or build diagnostics from strings that are later reused as identity. This leaves Stage 5/6 abstraction static verification short of the enum/newtype policy.

## 修正方針

Make TraitMethodCall and UnsatisfiedBound carry typed TraitApplication, derive diagnostic names only at the diagnostic boundary, remove infer_trait_application_name if it is no longer needed, and extend the abstraction source policy.

## 対応記録

- `TraitMethodCall` を `application: TraitApplication` payload へ移行し、`FuncRef::Trait` 生成時だけ HIR application へ変換する形にした。
- `TraitMethodResolution::UnsatisfiedBound` も `TraitApplication` を保持し、diagnostic 生成時だけ `display_name` を呼ぶようにした。
- `infer_trait_application_name` を削除し、trait method resolution の途中で表示名を作る経路をなくした。
- abstraction source policy に `TraitMethodCall` / `UnsatisfiedBound` の typed payload と split field 再導入禁止を追加し、`format_trait_ref_name` baseline を 6 から 4 へ下げた。

## 検証

- `cargo check -p nepl-core --tests`
- `cargo test -p nepl-core --test neplg2 trait -- --nocapture`
- `cargo test -p nepl-core --test neplg2 generic -- --nocapture`
- `cargo fmt --check -p nepl-core`
- `node nodesrc/test_abstraction_static_verification_policy.js`
- `node nodesrc/run_source_policy_regressions.js --warn-only`
- `node nodesrc/issues.js check --dir issues`
