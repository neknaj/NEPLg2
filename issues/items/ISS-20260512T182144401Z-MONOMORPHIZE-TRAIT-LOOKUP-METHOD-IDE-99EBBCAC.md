---
id: ISS-20260512T182144401Z-MONOMORPHIZE-TRAIT-LOOKUP-METHOD-IDE-99EBBCAC
title: "Monomorphize trait lookup method identity still uses raw String"
area: core
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-13
target: "nepl-core/src/monomorphize/trait_lookup.rs; nepl-core/src/monomorphize.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md"
---

# ISS-20260512T182144401Z-MONOMORPHIZE-TRAIT-LOOKUP-METHOD-IDE-99EBBCAC: Monomorphize trait lookup method identity still uses raw String

## 概要

MonoTraitMethodKey and MonoTraitLookupKey are named structs, but their method identity is still stored as raw String. This keeps method identity as an untyped display/name payload inside the lookup key and weakens the enum/newtype based static verification policy for trait abstraction.

## 対象

- `nepl-core/src/monomorphize/trait_lookup.rs`
- `nepl-core/src/monomorphize.rs`
- `nodesrc/test_abstraction_static_verification_policy.js`
- `doc/neplg2/abstraction_static_verification_plan.md`

## 根拠

- `doc/neplg2/abstraction_static_verification_plan.md` Stage 5 は monomorphize lookup key の method を `TraitMethodId` として扱う目標を明記している。
- `MonoTraitMethodKey` と `MonoTraitLookupKey` は named key だが、変更前は `method: String` を field として保持していた。
- method が raw `String` のままだと、trait lookup の key model は tuple からは脱していても method identity と表示名の境界を型で区別できない。
- 関連計画: [NEPLg2 abstraction static verification plan Stage 5](../../doc/neplg2/abstraction_static_verification_plan.md#stage-5-monomorphize-lookup-key-typed-%E5%8C%96)

## 問題

MonoTraitMethodKey and MonoTraitLookupKey are named structs, but their method identity is still stored as raw String. This keeps method identity as an untyped display/name payload inside the lookup key and weakens the enum/newtype based static verification policy for trait abstraction.

## 影響

Future monomorphize trait lookup changes can accidentally mix display method names with compiler identity keys. This is smaller than the old tuple-key issue, but it leaves Stage 5 short of the TraitMethodId/newtype boundary described in the abstraction plan.

## 修正方針

Introduce a typed MonoTraitMethodId newtype for monomorphize trait lookup keys, use it in MonoTraitMethodKey and MonoTraitLookupKey, and extend the abstraction source policy so raw method String fields cannot return to those key structs.

## 対応記録

- `MonoTraitMethodId` newtype を追加した。
- `MonoTraitMethodKey` と `MonoTraitLookupKey` の `method` field を `String` から `MonoTraitMethodId` へ移行した。
- `monomorphize.rs` は impl index と exact lookup cache の key 作成時に `MonoTraitMethodId::from_name` を通す。
- abstraction source policy に `MonoTraitMethodKey` / `MonoTraitLookupKey` が `method: String` を持たないことを追加した。

## 検証

cargo test -p nepl-core monomorphize::tests::public_monomorphize_returns_unresolved_trait_calls_without_panicking -- --nocapture; cargo test -p nepl-core --test neplg2 trait -- --nocapture; node nodesrc/test_abstraction_static_verification_policy.js; node nodesrc/issues.js check --dir issues

## 確認済み

- `cargo test -p nepl-core monomorphize::tests::public_monomorphize_returns_unresolved_trait_calls_without_panicking -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 trait -- --nocapture`: 18 passed
- `node nodesrc/test_abstraction_static_verification_policy.js`: pass
- `node nodesrc/test_parser_backend_responsibility_policy.js`: pass
