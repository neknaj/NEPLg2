---
id: ISS-20260512T152402849Z-TRAITBOUND-STILL-STORES-TRAIT-APPLIC-B9D13982
title: "TraitBound still stores trait application as split fields"
area: core
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-12
target: "nepl-core/src/typecheck/traits.rs; nepl-core/src/typecheck/function_check.rs; nepl-core/src/typecheck/trait_check.rs; nepl-core/src/typecheck/trait_bound_apply.rs; nodesrc/test_abstraction_static_verification_policy.js"
---

# ISS-20260512T152402849Z-TRAITBOUND-STILL-STORES-TRAIT-APPLIC-B9D13982: TraitBound still stores trait application as split fields

## 概要

After removing rendered bound names, TraitBound still keeps trait_base_name and trait_args as separate fields. Trait application is not a first-class typed value yet.

## 対象

- `nepl-core/src/typecheck/traits.rs; nepl-core/src/typecheck/function_check.rs; nepl-core/src/typecheck/trait_check.rs; nepl-core/src/typecheck/trait_bound_apply.rs; nodesrc/test_abstraction_static_verification_policy.js`

## 根拠

- `TraitBound` は表示名 field を削除した後も、`trait_base_name` と `trait_args` を別 field として持っていた。
- `function_check.rs`、`trait_check.rs`、`trait_bound_apply.rs` は base name と args を個別に渡していた。
- `doc/neplg2/abstraction_static_verification_plan.md` の Stage 1 は trait reference を typed value として保持する `TraitApplication` 導入を求めている。

## 問題

After removing rendered bound names, TraitBound still keeps trait_base_name and trait_args as separate fields. Trait application is not a first-class typed value yet.

## 影響

Call sites can pass base names and argument lists separately, making it easier to mismatch identity parts or reintroduce ad hoc string handling instead of an explicit TraitApplication model.

## 修正方針

Introduce TraitApplication, store it inside TraitBound, use it for bound display and matching, and add source policy checks that TraitBound owns a TraitApplication field.

## 対応記録

- `TraitApplication` を追加し、trait base name と type argument list を 1 つの typed value にまとめた。
- `TraitBound` は `application: TraitApplication` と `trait_self_ty` を保持する形にした。
- bound display と bound matching は `TraitApplication` / `TraitBound` の method 経由に寄せた。
- `nodesrc/test_abstraction_static_verification_policy.js` に `TraitApplication` の存在、`TraitBound.application`、`TraitBound` が split fields を持たないことを固定する検査を追加した。

## 検証

- `cargo test -p nepl-core generics -- --nocapture`
- `node nodesrc/tests.js -i tests/compiler/generic_impl_trait_args.n.md -i tests/compiler/generics.n.md --no-tree -o tmp/trait-application-struct-bound-generics.json -j 1 --dist web/dist`
- `node nodesrc/test_abstraction_static_verification_policy.js`
- `cargo check -p nepl-core --tests`
- `node nodesrc/run_source_policy_regressions.js --warn-only`
