---
id: ISS-20260513T162705742Z-TRAITSEMANTICS-STORES-CAPABILITY-TRA-C51DB134
title: "TraitSemantics stores capability trait identity with redundant raw names"
area: core
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-05-13
updated: 2026-05-13
target: "nepl-core/src/typecheck/traits.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md"
---

# ISS-20260513T162705742Z-TRAITSEMANTICS-STORES-CAPABILITY-TRA-C51DB134: TraitSemantics stores capability trait identity with redundant raw names

## 概要

TraitCapability is enum-based, but TraitSemantics stores copy/clone/drop capability traits as Vec<(String, TypeId)> even though all semantic checks use only TypeId. The retained raw trait names are redundant authority-adjacent state and can be misused by future abstraction or static verification changes.

## 対象

- `nepl-core/src/typecheck/traits.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md`

## 根拠

- `nepl-core/src/typecheck/traits.rs` の `TraitSemantics` は `TraitCapability` の検出結果を `copy_traits` / `clone_traits` / `drop_traits` に分けていたが、各要素は `(String, TypeId)` だった。
- `has_copy_capability` / `has_clone_capability` / `has_drop_capability` は `TypeId` しか参照しておらず、`String` は semantic authority として不要だった。
- 抽象化機能では trait identity を表示文字列ではなく typed value で保持する方針であり、capability trait 集合だけ raw name を残すと将来の判定が表示名に戻る抜け道になる。

## 問題

TraitCapability is enum-based, but TraitSemantics stores copy/clone/drop capability traits as Vec<(String, TypeId)> even though all semantic checks use only TypeId. The retained raw trait names are redundant authority-adjacent state and can be misused by future abstraction or static verification changes.

## 影響

Generic/trait capability checks can drift back toward rendered-name authority, weakening the typed abstraction model and making source policy less complete.

## 修正方針

Store capability trait identity as typed TypeId sets/lists only, keep capability category selection through exhaustive TraitCapability match, and extend abstraction source policy and docs so raw String capability identity is not reintroduced.

## 検証

- `cargo fmt --package nepl-core --check`: passed
- `node nodesrc/test_abstraction_static_verification_policy.js`: passed
- `cargo test -p nepl-core --test neplg2 trait -- --nocapture`: passed, 18 tests
- `node nodesrc/issues.js check --dir issues`: passed
- 参考: `node nodesrc/run_source_policy_regressions.js` は今回変更外の `resource/lower.rs` line limit 既存超過で停止したため、別 issue として分離する。

## 対応内容

- `TraitSemantics` の capability trait 集合を `Vec<(String, TypeId)>` から `Vec<TypeId>` へ変更した。
- capability category の選択は `insert_trait` の `match TraitCapability` に集約し、Copy / Clone / Drop の enum 網羅性が効く形にした。
- `nodesrc/test_abstraction_static_verification_policy.js` に `TraitSemantics` が raw `String` capability identity を保持しないことを固定する検査を追加した。
- `doc/neplg2/abstraction_static_verification_plan.md` の Stage 6 と進捗状況に capability semantics の typed identity 化を追記した。
