---
id: ISS-20260512T143721313Z-GENERIC-AND-TRAIT-ABSTRACTION-MODEL--1F2FF429
title: "Generic and trait abstraction model still uses string-rendered trait references"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-12
target: "nepl-core/src/typecheck/traits.rs; nepl-core/src/typecheck/trait_check.rs; nepl-core/src/typecheck/trait_bound_apply.rs; nepl-core/src/typecheck/trait_call_apply.rs; nepl-core/src/monomorphize.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md"
---

# ISS-20260512T143721313Z-GENERIC-AND-TRAIT-ABSTRACTION-MODEL--1F2FF429: Generic and trait abstraction model still uses string-rendered trait references

## 概要

Generics, trait bounds, impl lookup, and monomorphization have useful coverage and module splits, but core trait application identity is still partly represented through rendered strings such as TraitBoundRef.name, format_trait_ref_name, parse_trait_ref_name, and string-keyed monomorphize maps. parse_trait_ref_name only reconstructs primitive type arguments, and ImplInfo mixes inherent/trait impl state with Option<String> fields instead of a typed ImplKind/TraitApplication model.

## 対象

- `nepl-core/src/typecheck/traits.rs; nepl-core/src/typecheck/trait_check.rs; nepl-core/src/typecheck/trait_bound_apply.rs; nepl-core/src/typecheck/trait_call_apply.rs; nepl-core/src/monomorphize.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md`

## 根拠

- `nepl-core/src/typecheck/traits.rs` は `TraitBoundRef.name` を `format_trait_ref_name` で作り、`parse_trait_ref_name` で表示文字列から trait argument を復元する経路を持つ。
- `parse_trait_ref_name` は primitive type のみを復元するため、struct / enum / nested apply / type parameter を trait application identity の authority として扱えない。
- `ImplInfo` は `trait_name: Option<String>`、`trait_base_name: Option<String>`、`trait_self_ty: Option<TypeId>` を持ち、inherent impl と trait impl の状態を enum で分けていない。
- `monomorphize.rs` は trait lookup cache と impl index を trait / method の文字列 key と `Vec<TypeId>` で構成している。
- 詳細計画: [NEPLg2 abstraction static verification plan](../../doc/neplg2/abstraction_static_verification_plan.md)

## 問題

Generics, trait bounds, impl lookup, and monomorphization have useful coverage and module splits, but core trait application identity is still partly represented through rendered strings such as TraitBoundRef.name, format_trait_ref_name, parse_trait_ref_name, and string-keyed monomorphize maps. parse_trait_ref_name only reconstructs primitive type arguments, and ImplInfo mixes inherent/trait impl state with Option<String> fields instead of a typed ImplKind/TraitApplication model.

## 影響

Trait bound satisfaction and monomorphize lookup can drift between typecheck and backend lowering. String parsing and optional fields weaken static exhaustiveness, make non-primitive trait arguments fragile, and conflict with the enum/match-based static verification policy for type safety and capability checks.

## 修正方針

Introduce typed TraitApplication/TraitId/ImplKind/PendingTraitCheck/MonoTraitLookupKey structures, remove trait-reference string parsing from static-check internals, keep strings only at diagnostics/display boundaries, and add source policies/regressions that reject new string-based trait reference authority.

## 対応記録

- `doc/neplg2/abstraction_static_verification_plan.md` を追加し、現状評価、目標設計、Stage 0-6 の再設計計画を整理した。
- `nodesrc/test_abstraction_static_verification_policy.js` を追加し、`parse_trait_ref_name` / `format_trait_ref_name` / `TraitBoundRef` / `ImplInfo` / `trait_lookup_cache` の現行 baseline を固定した。
- `nodesrc/run_source_policy_regressions.js` に abstraction static verification policy を追加した。
- 現時点の baseline は最終状態ではなく、今後 `TraitApplication` / `ImplKind` / `PendingTraitCheck` / `MonoTraitLookupKey` 導入に合わせて 0 へ下げる。
- `ISS-20260512T145319534Z-DEFERRED-TRAIT-BOUND-CHECKS-REPARSE--38D11F7C` で、function-level deferred trait bound check が `bound.name` の表示文字列を authority として使う経路を除去した。`parse_trait_ref_name` baseline は 4 から 3 へ下げた。
- `ISS-20260512T150308333Z-TRAIT-METHOD-SELF-INFERENCE-REPARSES-FAE05801` で、trait method self inference が表示名を生成して parse し直す経路を除去した。`parse_trait_ref_name` は削除し、baseline は 0 にした。
- `ISS-20260512T151045280Z-TRAITBOUNDREF-STORES-RENDERED-DISPLA-644F8E6A` で、`TraitBoundRef` を `TraitBound` に改名し、rendered display name field を削除した。`TraitBoundRef` baseline は 0 にした。

## 検証

- `node nodesrc/test_abstraction_static_verification_policy.js`
- `node nodesrc/run_source_policy_regressions.js --warn-only`
- `node nodesrc/issues.js index --dir issues`
- `node nodesrc/issues.js check --dir issues`
