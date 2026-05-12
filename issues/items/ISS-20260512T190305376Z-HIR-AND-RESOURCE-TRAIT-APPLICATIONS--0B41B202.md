---
id: ISS-20260512T190305376Z-HIR-AND-RESOURCE-TRAIT-APPLICATIONS--0B41B202
title: "HIR and Resource trait applications still store trait identity as raw String"
area: core
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-13
target: "nepl-core/src/hir.rs; nepl-core/src/resource/model.rs; nepl-core/src/monomorphize.rs; nepl-core/src/monomorphize/trait_lookup.rs; nepl-core/src/resource/lower.rs; nepl-core/src/resource/dump.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md"
---

# ISS-20260512T190305376Z-HIR-AND-RESOURCE-TRAIT-APPLICATIONS--0B41B202: HIR and Resource trait applications still store trait identity as raw String

## 概要

After typecheck and monomorphize trait identity were moved to newtypes, HirTraitApplication and ResourceTraitApplication still store base_name: String. These intermediate representations are static-check and lowering authorities, so they can still pass rendered trait names by field convention.

## 対象

- `nepl-core/src/hir.rs`
- `nepl-core/src/resource/model.rs`
- `nepl-core/src/monomorphize.rs`
- `nepl-core/src/monomorphize/trait_lookup.rs`
- `nepl-core/src/resource/lower.rs`
- `nepl-core/src/resource/dump.rs`
- `nodesrc/test_abstraction_static_verification_policy.js`
- `doc/neplg2/abstraction_static_verification_plan.md`

## 根拠

- `doc/neplg2/abstraction_static_verification_plan.md` Stage 5 は HIR / Resource IR / monomorphize の trait application identity を typed model としてつなぐ方針を持つ。
- typecheck 側は `TraitId`、monomorphize 側は `MonoTraitId` へ移行済みだったが、変更前の `HirTraitApplication` と `ResourceTraitApplication` は `base_name: String` を保持していた。
- HIR と Resource IR は diagnostic 表示だけでなく monomorphize lookup と Resource lowering の入力であり、raw string identity field を残すと中間IR境界でtyped identityが途切れる。
- 関連計画: [NEPLg2 abstraction static verification plan Stage 5](../../doc/neplg2/abstraction_static_verification_plan.md#stage-5-monomorphize-lookup-key-typed-%E5%8C%96)

## 問題

After typecheck and monomorphize trait identity were moved to newtypes, HirTraitApplication and ResourceTraitApplication still store base_name: String. These intermediate representations are static-check and lowering authorities, so they can still pass rendered trait names by field convention.

## 影響

Trait identity can drift between typecheck, monomorphize, Resource IR, and diagnostics. Source policy proves that application structs exist, but not that their identity field is typed.

## 修正方針

Introduce HirTraitId and ResourceTraitId newtypes, store them in HirTraitApplication and ResourceTraitApplication, expose string names only through as_str/display boundaries, and extend abstraction source policy to reject base_name: String in HIR and Resource trait applications.

## 対応記録

- `HirTraitId` newtype を追加し、`HirTraitApplication` の trait identity を `base_name: String` から `trait_id: HirTraitId` へ移行した。
- `ResourceTraitId` newtype を追加し、`ResourceTraitApplication` の trait identity を `trait_id: ResourceTraitId` へ移行した。
- monomorphize と Resource IR lowering / dump は `as_str()` 境界でのみ trait name 文字列へ変換する。
- abstraction source policy に HIR / Resource trait application の typed id 必須と raw `base_name: String` 再導入禁止を追加した。

## 検証

cargo test -p nepl-core --test neplg2 trait -- --nocapture; cargo test -p nepl-core --test resource_ir trait -- --nocapture; cargo check -p nepl-core --tests; node nodesrc/test_abstraction_static_verification_policy.js; node nodesrc/issues.js check --dir issues

## 確認済み

- `cargo check -p nepl-core --tests`: pass
- `cargo test -p nepl-core --test neplg2 trait -- --nocapture`: 18 passed
- `cargo test -p nepl-core --test neplg2 generic -- --nocapture`: 10 passed
- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 249 passed
- `cargo fmt --check -p nepl-core`: pass
- `node nodesrc/test_abstraction_static_verification_policy.js`: pass
- `node nodesrc/run_source_policy_regressions.js --warn-only`: pass
- `node nodesrc/issues.js check --dir issues`: pass
