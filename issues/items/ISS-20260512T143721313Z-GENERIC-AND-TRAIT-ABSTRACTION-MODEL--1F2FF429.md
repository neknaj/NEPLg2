---
id: ISS-20260512T143721313Z-GENERIC-AND-TRAIT-ABSTRACTION-MODEL--1F2FF429
title: "Generic and trait abstraction model still uses string-rendered trait references"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-13
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
- `ISS-20260512T152402849Z-TRAITBOUND-STILL-STORES-TRAIT-APPLIC-B9D13982` で、`TraitApplication` を追加し、`TraitBound` が split fields ではなく typed application を持つようにした。
- `ISS-20260512T153756004Z-IMPLINFO-STILL-ENCODES-TRAIT-IMPL-ID-A4ECD77B` で、typecheck-side `ImplInfo` を `ImplKind` enum へ移行し、trait impl identity を `TraitApplication` と trait self `TypeId` で保持する形にした。`function_check.rs` / `trait_check.rs` / `trait_call_apply.rs` は `imp.trait_base_name` / `imp.trait_args` を直接読まない。
- `ISS-20260512T160137255Z-TRAIT-BOUND-LOOKUP-DUPLICATES-TYPEID-1694BE55` で、`trait_check.rs` の duplicate type parameter bound lookup を削除し、BlockChecker から `traits.rs` の typed helper へ委譲する形にした。
- `ISS-20260512T160950280Z-TRAIT-BOUND-LOOKUP-STILL-ACCEPTS-SAM-C03A85E0` で、typed helper 内の same-label `TypeId` fallback を削除し、bound lookup は exact / resolved `TypeId` と explicit substitution mapping に寄せた。
- `ISS-20260512T172241782Z-TRAIT-TYPE-PARAMETER-BOUNDS-STILL-EX-09CE8755` で、type parameter trait bounds を raw `BTreeMap<TypeId, Vec<TraitBound>>` ではなく `BoundEnv` に閉じ込め、`BlockChecker` / `BindingKind` / `check_function` の境界から raw map を削除した。
- `ISS-20260512T161908521Z-TRAIT-METHOD-RESOLUTION-STILL-RETURN-21525B05` で、trait method resolution を `TraitMethodResolution` enum と `TraitMethodCall` model に移し、selected callable / unbound call の分岐を enum match にした。
- `ISS-20260512T193917855Z-TRAIT-METHOD-RESOLUTION-STILL-CARRIE-0BFEEFA9` で、`TraitMethodCall` / `UnsatisfiedBound` の payload も `TraitApplication` へ移行し、trait method resolution 中に表示名を保持する経路を削除した。
- `ISS-20260512T163228542Z-PENDING-TRAIT-CHECKS-STILL-USE-POSIT-FB7F1082` で、pending trait bound check を tuple から `PendingTraitCheck { bound, target_ty, span }` へ移行した。
- `ISS-20260512T164311083Z-MONOMORPHIZE-TRAIT-LOOKUP-KEYS-STILL-DA66AC14` で、monomorphize の trait lookup key を tuple から `MonoTraitApplication` / `MonoTraitMethodKey` / `MonoTraitLookupKey` へ移行し、重複していた `impl_entry_index` を削除した。
- `ISS-20260512T165852784Z-HIR-TRAIT-CALLS-STILL-SPLIT-TRAIT-AP-405A462B` で、HIR 境界の `FuncRef::Trait` / `HirImpl` を `HirTraitApplication` へ移行し、trait identity の split string fields を削除した。
- `ISS-20260512T171317751Z-RESOURCE-IR-TRAIT-CALL-TARGET-STILL--6B70AE36` で、Resource IR の `ResourceCallTarget::Trait` を `ResourceTraitApplication` へ移行し、dump/report 境界でも trait application を named model として保持するようにした。
- `ISS-20260512T173702516Z-BOUNDENV-STILL-KEYS-TYPE-PARAMETER-B-792D9BA4` で、`BoundEnv` 内部 key を raw `TypeId` から `TypeParamId` newtype へ移行した。type parameter declaration identity は `BoundEnv::insert` / `iter` の境界でも明示され、source policy は raw `TypeId` key の再導入を拒否する。
- `ISS-20260512T174845757Z-ABSTRACTION-POLICY-COUNTS-TRAITINFO--FE3DB746` で、Stage 6 の source policy baseline を整理した。`ImplInfo` optional field は `traits.rs` 全体の `Option<String>` 数ではなく `ImplInfo` struct body を直接検査し、0 件を必須にした。
- `ISS-20260512T175900768Z-MONOMORPHIZE-TRAIT-LOOKUP-MODEL-EXCE-55B52B7E` で、monomorphize trait lookup key/model を `monomorphize/trait_lookup.rs` へ分離した。typed key model は維持し、abstraction policy は新 module を監視する。
- `ISS-20260512T155153362Z-GENERIC-TRAIT-PROBE-REGRESSION-FIXTU-65BB07DB` で、Stage 5 の trait/generic regression fixtures を修正した。raw memory を直接呼ぶ helper は impure signature にし、`PureCallsImpure` を弱めずに regression suite を復旧した。
- `ISS-20260512T182144401Z-MONOMORPHIZE-TRAIT-LOOKUP-METHOD-IDE-99EBBCAC` で、monomorphize trait lookup の method identity を `MonoTraitMethodId` newtype へ移行した。`MonoTraitMethodKey` / `MonoTraitLookupKey` は raw `String` method field を持たない。
- `ISS-20260512T183111826Z-MONOMORPHIZE-TRAIT-APPLICATION-STILL-835C27CF` で、monomorphize trait lookup の trait identity を `MonoTraitId` newtype へ移行した。`MonoTraitApplication` / `MonoTraitMethodKey` は raw trait `String` field を持たず、identity type は `monomorphize/trait_identity.rs` に分離した。
- `ISS-20260512T185123437Z-TYPECHECK-TRAITAPPLICATION-STILL-STO-F6F9CDD1` で、typecheck-side `TraitApplication` の trait identity を `TraitId` newtype へ移行した。`BoundEnv` / impl matching / trait method inference は `TraitId` を受け取り、表示文字列は diagnostic/display 境界に限定した。
- `ISS-20260512T190305376Z-HIR-AND-RESOURCE-TRAIT-APPLICATIONS--0B41B202` で、HIR / Resource IR の trait application identity を `HirTraitId` / `ResourceTraitId` newtype へ移行した。monomorphize と Resource lowering は `as_str()` 境界でのみ文字列化する。
- `ISS-20260512T191325765Z-HIR-AND-RESOURCE-TRAIT-METHOD-IDENTI-78952D7B` で、HIR / Resource IR の trait method identity を `HirTraitMethodId` / `ResourceTraitMethodId` newtype へ移行した。`FuncRef::Trait` / `ResourceCallTarget::Trait` は raw `String` method field を持たない。

## 検証

- `node nodesrc/test_abstraction_static_verification_policy.js`
- `node nodesrc/run_source_policy_regressions.js --warn-only`
- `node nodesrc/issues.js index --dir issues`
- `node nodesrc/issues.js check --dir issues`
