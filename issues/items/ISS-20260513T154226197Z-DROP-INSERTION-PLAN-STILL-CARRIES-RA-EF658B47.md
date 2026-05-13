---
id: ISS-20260513T154226197Z-DROP-INSERTION-PLAN-STILL-CARRIES-RA-EF658B47
title: "Drop insertion plan still carries raw trait names"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-13
updated: 2026-05-14
target: "nepl-core/src/passes/drop_insertion.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md"
---

# ISS-20260513T154226197Z-DROP-INSERTION-PLAN-STILL-CARRIES-RA-EF658B47: Drop insertion plan still carries raw trait names

## 概要

HIR trait call identity has been moved to HirTraitApplication/HirTraitMethodId, but DropPlan still stores trait_name and method_name as String and constructs FuncRef::Trait from raw names. This leaves auto-drop generation outside the typed abstraction model.

## 対象

- `nepl-core/src/passes/drop_insertion.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md`

## 根拠

- `doc/neplg2/abstraction_static_verification_plan.md` Stage 5/6 は、typecheck / HIR / monomorphize / Resource IR の trait application と method identity を typed model で接続し、表示文字列は diagnostic/display 境界だけに限定する方針である。
- `FuncRef::Trait` は `HirTraitApplication` / `HirTraitMethodId` を保持するように移行済みだったが、HIR drop insertion の `DropPlan` は `trait_name: String` と `method_name: String` を保持していた。
- `drop_call_expr` / `drop_field_call_expr` は `plan.trait_name` / `plan.method_name` から generated Drop call の `FuncRef::Trait` を組み立てていた。

## 問題

HIR trait call identity has been moved to HirTraitApplication/HirTraitMethodId, but DropPlan still stores trait_name and method_name as String and constructs FuncRef::Trait from raw names. This leaves auto-drop generation outside the typed abstraction model.

## 影響

A future drop insertion change can reintroduce rendered trait names as static-check authority while abstraction policy still passes, weakening enum/newtype coverage for generated Drop trait calls.

## 修正方針

Make DropPlan store HirTraitApplication and HirTraitMethodId, construct them when the Drop-capability trait is selected, and extend abstraction source policy to reject raw trait_name/method_name fields in DropPlan.

## 対応

- `DropPlan` を `trait_application: HirTraitApplication` と `method_id: HirTraitMethodId` を持つ形へ変更した。
- Drop-capability trait を選択する `find_drop_plan` で typed HIR payload を構築し、generated Drop call は `DropPlan` の typed payload を clone して `FuncRef::Trait` に渡すようにした。
- `nodesrc/test_abstraction_static_verification_policy.js` に `DropPlan` の typed payload 必須検査と、`plan.trait_name` / `plan.method_name` 再導入禁止検査を追加した。
- `doc/neplg2/abstraction_static_verification_plan.md` の Stage 5 進捗に、drop insertion の typed trait identity 化を追記した。

## 検証

- `cargo fmt --package nepl-core --check`: pass
- `node nodesrc/test_abstraction_static_verification_policy.js`: pass
- `cargo test -p nepl-core --test drop generic_result_enum_payload_auto_drop_uses_applied_type -- --nocapture`: pass
- `cargo test -p nepl-core --test drop auto_drop_runs_at_scope_end -- --nocapture`: pass
- 参考: `cargo test -p nepl-core --test drop auto_drop_partially_moved_struct_drops_remaining_fields -- --nocapture` は clean `main` でも既存 issue `ISS-20260429T231116550Z-AUTO-DROP-SKIPS-REMAINING-STRUCT-FIE-67E6E6C5` と同じ失敗を再現するため、今回の変更由来ではない。
- 参考: `cargo test -p nepl-core --test neplg2 generic -- --nocapture` と `cargo test -p nepl-core --test neplg2 trait -- --nocapture` は clean `main` でも raw memory boundary / trait fixture regression 系の既存失敗を含むため、今回の focused verification からは除外した。
