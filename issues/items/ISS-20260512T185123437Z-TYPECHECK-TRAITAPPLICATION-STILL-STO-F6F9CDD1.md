---
id: ISS-20260512T185123437Z-TYPECHECK-TRAITAPPLICATION-STILL-STO-F6F9CDD1
title: "Typecheck TraitApplication still stores trait identity as raw String"
area: core
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-13
target: "nepl-core/src/typecheck/traits.rs; nepl-core/src/typecheck/driver.rs; nepl-core/src/typecheck/block_check.rs; nepl-core/src/typecheck/prefix_check.rs; nepl-core/src/typecheck/trait_check.rs; nepl-core/src/typecheck/trait_bound_apply.rs; nepl-core/src/typecheck/trait_call_apply.rs; nepl-core/src/typecheck/function_check.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md"
---

# ISS-20260512T185123437Z-TYPECHECK-TRAITAPPLICATION-STILL-STO-F6F9CDD1: Typecheck TraitApplication still stores trait identity as raw String

## 概要

Typecheck-side TraitApplication has moved trait arguments into a typed model, but the trait identity itself is still stored as base_name: String. Static-check internals can still pass display trait names as authority by convention instead of through a typed TraitId boundary.

## 対象

- `nepl-core/src/typecheck/traits.rs`
- `nepl-core/src/typecheck/driver.rs`
- `nepl-core/src/typecheck/block_check.rs`
- `nepl-core/src/typecheck/prefix_check.rs`
- `nepl-core/src/typecheck/trait_check.rs`
- `nepl-core/src/typecheck/trait_bound_apply.rs`
- `nepl-core/src/typecheck/trait_call_apply.rs`
- `nepl-core/src/typecheck/function_check.rs`
- `nodesrc/test_abstraction_static_verification_policy.js`
- `doc/neplg2/abstraction_static_verification_plan.md`

## 根拠

- `doc/neplg2/abstraction_static_verification_plan.md` Stage 1 は `TraitApplication` / `TraitId` の導入を目標としている。
- Stage 1 の前回対応で trait args は `TraitApplication` にまとまったが、変更前の `TraitApplication` は `base_name: String` を保持していた。
- `BoundEnv::has_trait_application_bound`、`ImplInfo::matches_trait_application`、`infer_unique_type_param_for_trait_ref` は trait identity を `&str` で受け取り、typed model の内部でも表示名相当の文字列を authority として渡せた。
- 関連計画: [NEPLg2 abstraction static verification plan Stage 1](../../doc/neplg2/abstraction_static_verification_plan.md#stage-1-typed-traitapplication-%E5%B0%8E%E5%85%A5)

## 問題

Typecheck-side TraitApplication has moved trait arguments into a typed model, but the trait identity itself is still stored as base_name: String. Static-check internals can still pass display trait names as authority by convention instead of through a typed TraitId boundary.

## 影響

Trait bound satisfaction and impl matching remain weaker than the enum/newtype development policy. Future edits can mix diagnostic/rendered names with compiler lookup identity while source policy only checks that TraitApplication exists.

## 修正方針

Introduce a TraitId newtype for typecheck TraitApplication, require TraitApplication and BoundEnv trait-bound lookup to use TraitId, expose string lookup only through an explicit as_str boundary for existing declaration maps, and extend abstraction source policy to reject base_name: String inside TraitApplication.

## 対応記録

- `TraitId` newtype を追加し、`TraitApplication` の trait identity を `base_name: String` から `trait_id: TraitId` へ移行した。
- `TraitApplication::display_name` だけが `TraitId::as_str()` で diagnostic/display 文字列へ変換するようにした。
- `BoundEnv::has_trait_application_bound`、`type_param_has_trait_application_bound`、`ImplInfo::matches_trait_application`、`infer_unique_type_param_for_trait_ref` は `TraitId` を受け取る形にした。
- trait bound collection、impl collection、nested function bound collection、trait method inference は入口で `TraitId::from_name` を通す。
- abstraction source policy に `TraitId` 必須と `TraitApplication.base_name: String` 再導入禁止を追加した。

## 検証

cargo test -p nepl-core --test neplg2 trait -- --nocapture; cargo check -p nepl-core --tests; node nodesrc/test_abstraction_static_verification_policy.js; node nodesrc/issues.js check --dir issues

## 確認済み

- `cargo check -p nepl-core --tests`: pass
- `cargo test -p nepl-core --test neplg2 trait -- --nocapture`: 18 passed
- `cargo test -p nepl-core --test neplg2 generic -- --nocapture`: 10 passed
- `cargo fmt --check -p nepl-core`: pass
- `node nodesrc/test_abstraction_static_verification_policy.js`: pass
- `node nodesrc/run_source_policy_regressions.js --warn-only`: pass
