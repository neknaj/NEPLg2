---
id: ISS-20260512T183111826Z-MONOMORPHIZE-TRAIT-APPLICATION-STILL-835C27CF
title: "Monomorphize trait application still exposes trait identity as raw String"
area: core
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-13
target: "nepl-core/src/monomorphize/trait_identity.rs; nepl-core/src/monomorphize/trait_lookup.rs; nepl-core/src/monomorphize.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md"
---

# ISS-20260512T183111826Z-MONOMORPHIZE-TRAIT-APPLICATION-STILL-835C27CF: Monomorphize trait application still exposes trait identity as raw String

## 概要

MonoTraitApplication and MonoTraitMethodKey still expose trait identity as raw String fields. Even after tuple keys and method id were typed, monomorphize trait lookup can still mix display trait base names with compiler lookup identity by field convention.

## 対象

- `nepl-core/src/monomorphize/trait_identity.rs`
- `nepl-core/src/monomorphize/trait_lookup.rs`
- `nepl-core/src/monomorphize.rs`
- `nodesrc/test_abstraction_static_verification_policy.js`
- `nodesrc/test_parser_backend_responsibility_policy.js`
- `doc/neplg2/abstraction_static_verification_plan.md`
- `doc/neplg2/parser_backend_responsibility_split_plan.md`

## 根拠

- `doc/neplg2/abstraction_static_verification_plan.md` Stage 5 は monomorphize lookup key を typed identity に移す方針を明記している。
- `MonoTraitMethodId` 導入後も、`MonoTraitApplication` と `MonoTraitMethodKey` は trait identity を `String` field として保持していた。
- trait identity が raw `String` のままだと、typed key model の内部で表示名と compiler lookup identity を field convention だけで区別する状態が残る。
- `trait_lookup.rs` へ identity newtype を直接積み足すと責務分割 policy の 90 lines limit を越えるため、identity type は専用 module に分離する必要がある。
- 関連計画: [NEPLg2 abstraction static verification plan Stage 5](../../doc/neplg2/abstraction_static_verification_plan.md#stage-5-monomorphize-lookup-key-typed-%E5%8C%96)

## 問題

MonoTraitApplication and MonoTraitMethodKey still expose trait identity as raw String fields. Even after tuple keys and method id were typed, monomorphize trait lookup can still mix display trait base names with compiler lookup identity by field convention.

## 影響

Trait impl indexing and lookup cache remain weaker than the TraitId/newtype boundary required by the abstraction plan. Future edits can reintroduce string-keyed trait identity inside monomorphize while source policy only checks named key structs.

## 修正方針

Introduce a MonoTraitId newtype for monomorphize trait lookup, store it in MonoTraitApplication and MonoTraitMethodKey, expose display/name access only through an explicit as_str boundary, and extend abstraction source policy to reject raw String trait identity fields in those lookup structs.

## 対応記録

- `monomorphize/trait_identity.rs` を追加し、`MonoTraitId` と `MonoTraitMethodId` newtype を集約した。
- `MonoTraitApplication` の trait identity を `base_name: String` から `trait_id: MonoTraitId` に変更した。
- `MonoTraitMethodKey` の trait identity を `trait_base_name: String` から `trait_id: MonoTraitId` に変更した。
- `monomorphize.rs` の impl index / lookup cache 作成時に `MonoTraitId::from_name` を通すようにした。
- abstraction source policy は identity type が `trait_identity.rs` にあり、lookup key が raw trait `String` field を持たないことを検査する。
- parser/backend responsibility policy は `trait_identity.rs` を 45 lines 上限で監視し、`trait_lookup.rs` の line limit は上げない。

## 検証

cargo test -p nepl-core monomorphize::tests::public_monomorphize_returns_unresolved_trait_calls_without_panicking -- --nocapture; cargo test -p nepl-core --test neplg2 trait -- --nocapture; node nodesrc/test_abstraction_static_verification_policy.js; node nodesrc/test_parser_backend_responsibility_policy.js; node nodesrc/issues.js check --dir issues

## 確認済み

- `cargo check -p nepl-core --tests`: pass
- `cargo test -p nepl-core monomorphize::tests::public_monomorphize_returns_unresolved_trait_calls_without_panicking -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 trait -- --nocapture`: 18 passed
- `cargo fmt --check -p nepl-core`: pass
- `node nodesrc/test_abstraction_static_verification_policy.js`: pass
- `node nodesrc/test_parser_backend_responsibility_policy.js`: pass
- `node nodesrc/run_source_policy_regressions.js --warn-only`: pass
