---
id: ISS-20260512T164311083Z-MONOMORPHIZE-TRAIT-LOOKUP-KEYS-STILL-DA66AC14
title: "Monomorphize trait lookup keys still use positional tuple state"
area: core
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-13
target: "nepl-core/src/monomorphize.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md"
---

# ISS-20260512T164311083Z-MONOMORPHIZE-TRAIT-LOOKUP-KEYS-STILL-DA66AC14: Monomorphize trait lookup keys still use positional tuple state

## 概要

Stage 5 の monomorphize trait lookup は impl_map / impl_method_index / impl_entry_index / trait_lookup_cache を tuple key で保持しており、trait name / method / trait args / self type の意味を Rust の型検査で区別できない。

## 対象

- `nepl-core/src/monomorphize.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md`

## 根拠

- `nepl-core/src/monomorphize.rs` は `impl_map: BTreeMap<(String, String, TypeId), usize>` を使い、trait 名、method 名、self type を tuple の位置だけで区別していた。
- `impl_method_index` / `impl_entry_index` も `(String, String)` key を使い、applied trait 名と base trait 名の用途が map 名と呼び出し順に依存していた。
- `trait_lookup_cache` は `(String, String, Vec<TypeId>, TypeId)` key であり、trait args と self type の意味を named field で表していなかった。
- この状態は `doc/neplg2/abstraction_static_verification_plan.md` Stage 5 の typed monomorphize lookup 方針に反していた。

## 問題

Stage 5 の monomorphize trait lookup は impl_map / impl_method_index / impl_entry_index / trait_lookup_cache を tuple key で保持しており、trait name / method / trait args / self type の意味を Rust の型検査で区別できない。

## 影響

tuple の field order を取り違えても型が通るため、typecheck と monomorphize の trait application agreement が崩れ、generic trait impl resolution や Resource IR 後段の安全検査が誤った関数へ接続される危険がある。

## 修正方針

MonoTraitApplication / MonoTraitMethodKey / MonoTraitLookupKey を導入し、impl index と lookup cache を named key model へ移す。source policy で tuple key の再導入を拒否する。

## 対応記録

- `MonoTraitApplication` を追加し、monomorphize 内部の trait application を `base_name` と `args` の named field で保持するようにした。
- `MonoTraitMethodKey` を追加し、trait base name と method の候補 index を named key へ移行した。
- `MonoTraitLookupKey` を追加し、exact impl lookup と lookup cache を trait application / method / self type の named key へ移行した。
- `impl_entry_index` は `impl_method_index` と同じ候補集合を二重管理していたため削除し、base trait + method の候補 index に統合した。
- `nodesrc/test_abstraction_static_verification_policy.js` に monomorphize tuple key の再導入禁止を追加し、`traitLookupCache` baseline を 6 に下げた。
- `doc/neplg2/abstraction_static_verification_plan.md` Stage 5 に進捗と残件を追記した。

## 検証

cargo check -p nepl-core --tests; cargo test -p nepl-core monomorphize::tests::public_monomorphize_returns_unresolved_trait_calls_without_panicking -- --nocapture; cargo test -p nepl-core --test neplg2 generic_trait_impl_method_resolves_by_trait_args -- --nocapture; node nodesrc/test_abstraction_static_verification_policy.js

## 確認済み

- `cargo check -p nepl-core --tests`: pass
- `node nodesrc/test_abstraction_static_verification_policy.js`: pass
- `cargo test -p nepl-core monomorphize::tests::public_monomorphize_returns_unresolved_trait_calls_without_panicking -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 generic_trait_impl_method_resolves_by_trait_args -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 trait -- --nocapture`: 17/18 pass。失敗 1 件は既知 open issue `ISS-20260512T155153362Z-GENERIC-TRAIT-PROBE-REGRESSION-FIXTU-65BB07DB` の `Effect(PureCallsImpure)` fixture で、lookup key 変更とは別問題。
