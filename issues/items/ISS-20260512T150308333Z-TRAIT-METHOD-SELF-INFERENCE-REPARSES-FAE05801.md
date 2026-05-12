---
id: ISS-20260512T150308333Z-TRAIT-METHOD-SELF-INFERENCE-REPARSES-FAE05801
title: "Trait method self inference reparses rendered trait applications"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-05-12
updated: 2026-05-12
target: "nepl-core/src/typecheck/prefix_check.rs; nepl-core/src/typecheck/trait_check.rs; nepl-core/src/typecheck/traits.rs; nodesrc/test_abstraction_static_verification_policy.js"
---

# ISS-20260512T150308333Z-TRAIT-METHOD-SELF-INFERENCE-REPARSES-FAE05801: Trait method self inference reparses rendered trait applications

## 概要

Trait method self type inference in prefix_check.rs formats inferred trait arguments into a rendered trait name and then infer_unique_type_param_for_trait reparses that display string. This keeps parse_trait_ref_name in static-check internals.

## 対象

- `nepl-core/src/typecheck/prefix_check.rs; nepl-core/src/typecheck/trait_check.rs; nepl-core/src/typecheck/traits.rs; nodesrc/test_abstraction_static_verification_policy.js`

## 根拠

- `prefix_check.rs` は trait method を callable stack entry として積むとき、`infer_trait_application_name` で表示名を作ってから `infer_unique_type_param_for_trait` に渡していた。
- `infer_unique_type_param_for_trait` は `parse_trait_ref_name` で表示文字列から trait argument を復元していた。
- 同じ時点で `infer_trait_application_args` は typed `TypeId` 列を返せるため、表示名を経由する必要がなかった。

## 問題

Trait method self type inference in prefix_check.rs formats inferred trait arguments into a rendered trait name and then infer_unique_type_param_for_trait reparses that display string. This keeps parse_trait_ref_name in static-check internals.

## 影響

Trait method inference can lose non-primitive, nested, or type-parameter trait arguments, and static verification still depends on diagnostic/display text instead of typed trait application data.

## 修正方針

Pass inferred trait argument TypeId values directly into infer_unique_type_param_for_trait_ref, remove infer_unique_type_param_for_trait, delete parse_trait_ref_name, and lower the source policy baseline to zero.

## 対応記録

- `prefix_check.rs` は `infer_trait_application_args` の戻り値を直接 `infer_unique_type_param_for_trait_ref` へ渡すようにした。
- `trait_check.rs` から `infer_unique_type_param_for_trait` を削除した。
- `traits.rs` から `parse_trait_ref_name` を削除した。
- `nodesrc/test_abstraction_static_verification_policy.js` の `parse_trait_ref_name` baseline を 0 にし、再導入禁止を固定した。

## 検証

- `cargo test -p nepl-core generics -- --nocapture`
- `node nodesrc/tests.js -i tests/compiler/generic_impl_trait_args.n.md -i tests/compiler/generics.n.md --no-tree -o tmp/typed-trait-method-self-inference-generics.json -j 1 --dist web/dist`
- `node nodesrc/test_abstraction_static_verification_policy.js`
- `cargo check -p nepl-core --tests`
- `node nodesrc/run_source_policy_regressions.js --warn-only`
