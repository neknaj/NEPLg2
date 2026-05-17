---
id: ISS-20260517T173445922Z-RESOURCE-IR-FIELD-PROJECTION-PROOF-A-7ABBA4D1
title: "Resource IR field projection proof accepts ordinary get direct calls"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-17
updated: 2026-05-18
target: "nepl-core/src/resource/lower.rs, nepl-core/src/resource/lower_aggregate.rs, nepl-core/src/resource/coverage_hir*.rs, nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260517T173445922Z-RESOURCE-IR-FIELD-PROJECTION-PROOF-A-7ABBA4D1: Resource IR field projection proof accepts ordinary get direct calls

## 概要

Resource IR lowering and coverage classify any direct call whose base name is get/get_ref as a field accessor projection. This bypasses the typecheck field accessor proof and can make ordinary user functions named get/get_ref look like aggregate field reads.

## 対象

- `nepl-core/src/resource/lower.rs, nepl-core/src/resource/lower_aggregate.rs, nepl-core/src/resource/coverage_hir*.rs, nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- `nepl-core/src/resource/lower.rs` は `FuncRef` の base name が `get` / `get_ref` に見える direct call を、typecheck 済みの field accessor HIR かどうかを見ずに `ResourceOp::Read` / `ResourceOp::Borrow` へ lower していた。
- `nepl-core/src/resource/coverage_hir.rs` / `coverage_hir_projection.rs` も同じ direct call spelling を coverage 上の field read として数えていた。
- `lower.rs` の `direct_call_needs_recursive_lowering` は `FieldAccessorKind::from_call_base_name` で ordinary direct call を再分類しており、typecheck が生成した field accessor intrinsic evidence と普通の user function 名を分離できていなかった。

## 問題

Resource IR lowering and coverage classify any direct call whose base name is get/get_ref as a field accessor projection. This bypasses the typecheck field accessor proof and can make ordinary user functions named get/get_ref look like aggregate field reads.

## 影響

Static resource checking can be driven by function spelling instead of typed HIR/intrinsic evidence. This weakens the enum-first proof discipline and makes checker bugs harder to detect.

## 修正方針

Only field accessor intrinsics or typecheck-proven field accessor HIR should count as field projections. Remove normal FuncRef get/get_ref classification from Resource IR lowering and coverage, then add policy/regression coverage.

## 検証

Run focused resource/static policy tests and cargo test for nepl-core resource/typecheck affected modules.

## 関連計画

- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6: Resource IR は ordinary `FuncRef` の spelling ではなく、typecheck が生成した typed HIR / intrinsic evidence を field projection proof として消費する。

## 修正内容

- Resource IR coverage から ordinary `FuncRef` の `get` / `get_ref` direct call を field read として数える経路を削除した。
- Resource IR lowering から ordinary `FuncRef` の `get` / `get_ref` direct call を `ResourceOp::Read` / `ResourceOp::Borrow` へ変換する経路を削除した。
- direct call recursion gate が `FieldAccessorKind::from_call_base_name` で ordinary callee spelling を field accessor として再分類しないようにした。
- `nodesrc/test_static_check_boundary_responsibility.js` に、Resource IR が ordinary direct call 名から field projection proof を作らないことを確認する source policy を追加した。
- ordinary user function `get(Pair, str)` が Resource IR で `Call` のまま残り、field projection `Read` にならない regression test を追加した。

## 検証結果

- `cargo fmt -p nepl-core --check`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core resource::lower::tests::ordinary_get_direct_call_is_not_field_projection -- --nocapture`: 1 passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
