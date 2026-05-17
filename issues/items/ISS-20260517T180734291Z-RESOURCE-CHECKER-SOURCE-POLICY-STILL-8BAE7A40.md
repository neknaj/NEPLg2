---
id: ISS-20260517T180734291Z-RESOURCE-CHECKER-SOURCE-POLICY-STILL-8BAE7A40
title: "resource checker source policy still expects removed field accessor name classifier"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-17
updated: 2026-05-17
target: "nodesrc/test_resource_checker_responsibility.js, nepl-core/src/resource/lower.rs, nepl-core/src/resource/lower_tests.rs, doc/neplg2/static_check_complexity_reduction_plan.md"
---

# ISS-20260517T180734291Z-RESOURCE-CHECKER-SOURCE-POLICY-STILL-8BAE7A40: resource checker source policy still expects removed field accessor name classifier

## 概要

nodesrc/test_resource_checker_responsibility.js still requires FieldAccessorKind::from_call_base_name in resource/lower.rs, even though direct call-name based field accessor classification was removed to avoid treating ordinary get/get_ref calls as proof evidence.

## 対象

- `nodesrc/test_resource_checker_responsibility.js, nepl-core/src/resource/lower.rs`

## 根拠

- `ISS-20260517T173445922Z-RESOURCE-IR-FIELD-PROJECTION-PROOF-A-7ABBA4D1` と `ISS-20260517T175341624Z-TRANSPARENT-RAW-ADDRESS-RETURN-PROOF-52AEEF7B` で、ordinary direct call 名から field accessor proof を推測する経路は削除済みである。
- 現行の正しい設計では、Resource IR の field projection proof は typecheck が生成した intrinsic / typed HIR evidence を読む。ordinary `get` / `get_ref` 関数名は proof にならない。
- `nodesrc/test_resource_checker_responsibility.js` だけが古い `FieldAccessorKind::from_call_base_name` 要求を残しており、source policy registry 全体を失敗させていた。
- 同じ確認中に、recent regression tests が `lower.rs` へ直置きされ、Resource checker 責務分割の行数上限を超えていることも露出した。

## 問題

nodesrc/test_resource_checker_responsibility.js still requires FieldAccessorKind::from_call_base_name in resource/lower.rs, even though direct call-name based field accessor classification was removed to avoid treating ordinary get/get_ref calls as proof evidence.

## 影響

Running the source policy registry fails on main and can pressure developers to reintroduce an unsound name-based classifier instead of the Resource IR/source-capability proof boundary.

## 修正方針

Rewrite the source policy to enforce the new proof model: ordinary direct calls must not be classified as field accessors, transparent raw-address return proof evidence must remain explicit, and the removed from_call_base_name classifier must stay absent.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js and node nodesrc/run_source_policy_regressions.js after the policy update.

## 対応結果

- `nodesrc/test_resource_checker_responsibility.js` の古い `FieldAccessorKind::from_call_base_name` 存在要求を削除し、再導入禁止へ反転した。
- transparent raw-address return proof は `RawAddressReturnCalleeEvidence` を確認対象にし、`OrdinaryCall => None` と `Intrinsic => FieldAccessorKind::from_intrinsic_name(...)` の分離を policy として固定した。
- coverage 側の stale 関数名 `field_get_call_owner` も、現行の `get_field_intrinsic_owner` / `get_field_ref_intrinsic_owner` に合わせた。
- `lower.rs` に残っていた regression tests を `lower_tests.rs` へ分離し、Resource IR lowering 本体の責務分割上限を守る形にした。
- `doc/neplg2/static_check_complexity_reduction_plan.md` に Stage 6 の proof policy 同期として記録した。

検証:

- `cargo fmt -p nepl-core --check`: pass
- `cargo check -p nepl-core --tests`: pass
- `cargo test -p nepl-core resource::lower::tests -- --nocapture`: pass
- `node nodesrc/test_resource_checker_responsibility.js`: pass
- `node nodesrc/test_static_check_boundary_responsibility.js`: pass
- `node nodesrc/run_source_policy_regressions.js`: pass
