---
id: ISS-20260512T194845369Z-IMPL-METHOD-LOWERING-STILL-KEEPS-REN-E15DE8F5
title: "Impl method lowering still keeps rendered trait application name"
area: core
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-13
target: "nepl-core/src/typecheck/driver.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md"
---

# ISS-20260512T194845369Z-IMPL-METHOD-LOWERING-STILL-KEEPS-REN-E15DE8F5: Impl method lowering still keeps rendered trait application name

## 概要

The second impl lowering pass builds applied_trait_name with format_trait_ref_name and keeps it as a local payload before method mangling and HIR impl construction. The impl identity model is typed, but this pass still carries rendered trait application text beside trait_name and trait_args.

## 対象

- `nepl-core/src/typecheck/driver.rs; nodesrc/test_abstraction_static_verification_policy.js; doc/neplg2/abstraction_static_verification_plan.md`

## 根拠

- `doc/neplg2/abstraction_static_verification_plan.md` Stage 5/6 は impl / monomorphize / HIR の trait application identity を typed value で共有し、表示名を identity authority にしない方針である。
- 2 回目の impl lowering pass だけが `format_trait_ref_name` を直接呼び、`applied_trait_name` を method symbol mangle と HIR impl construction の手前で保持していた。

## 問題

The second impl lowering pass builds applied_trait_name with format_trait_ref_name and keeps it as a local payload before method mangling and HIR impl construction. The impl identity model is typed, but this pass still carries rendered trait application text beside trait_name and trait_args.

## 影響

A future impl lowering change can reuse the rendered name as identity or diverge from ImplInfo's typed TraitApplication. This keeps Stage 5/6 abstraction verification partially dependent on display text.

## 修正方針

Build a TraitApplication in the lowering pass, derive the mangling display name only at the symbol boundary, construct HirTraitApplication from the typed application, and extend the abstraction source policy.

## 対応記録

- impl method lowering pass で `TraitApplication` を構築し、`trait_args` split payload を持ち回らない形にした。
- method symbol の mangle 直前だけ `trait_application.display_name(&ctx)` を呼ぶようにし、`applied_trait_name` payload と direct `format_trait_ref_name` import を削除した。
- final `HirImpl` の `HirTraitApplication` も typed `TraitApplication` から変換する形にした。
- abstraction source policy に `driver.rs` の direct `format_trait_ref_name` / `applied_trait_name` 再導入禁止を追加し、`format_trait_ref_name` baseline を 4 から 2 へ下げた。

## 検証

- `cargo check -p nepl-core --tests`
- `cargo test -p nepl-core --test neplg2 trait -- --nocapture`
- `cargo test -p nepl-core --test neplg2 generic -- --nocapture`
- `cargo fmt --check -p nepl-core`
- `node nodesrc/test_abstraction_static_verification_policy.js`
- `node nodesrc/run_source_policy_regressions.js --warn-only`
- `node nodesrc/issues.js check --dir issues`
