---
id: ISS-20260506T084621972Z-RESOURCE-IR-LIVE-DROP-FACTS-LACK-COD-9EB91BC5
title: "Resource IR live drop facts lack codegen-facing elaboration plan"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/src/resource, nepl-core/src/compiler.rs, nepl-core/src/passes/drop_insertion.rs"
---

# ISS-20260506T084621972Z-RESOURCE-IR-LIVE-DROP-FACTS-LACK-COD-9EB91BC5: Resource IR live drop facts lack codegen-facing elaboration plan

## 概要

ResourceFunctionCheck::auto_drop_points now records live initialized auto-drop facts, but there is no validated codegen-facing plan that consumes those checked facts. The remaining drop insertion path can still be wired to candidate drop points or HIR scope traversal by mistake.

## 対象

- `nepl-core/src/resource, nepl-core/src/compiler.rs, nepl-core/src/passes/drop_insertion.rs`

## 根拠

- 未記入

## 問題

ResourceFunctionCheck::auto_drop_points now records live initialized auto-drop facts, but there is no validated codegen-facing plan that consumes those checked facts. The remaining drop insertion path can still be wired to candidate drop points or HIR scope traversal by mistake.

## 影響

Drop elaboration migration can accidentally reintroduce double-drop or missing-drop risks by consuming ResourceDropPlan candidates instead of checked live facts, leaving HIR scope walker as an implicit second authority.

## 修正方針

Add an explicit ResourceDropElaborationPlan built from ResourceCheckReport auto_drop_points, validate each path against Resource IR EndScope anchors, reject mismatched function/check/path/locals with typed enum errors, and use it as the boundary for the next codegen migration step.

## 検証

Focused Resource IR regression should prove that moved locals are absent from the elaboration plan, parameter drop points resolve to EndScope anchors, invalid paths are rejected by enum errors, plus source policy and issue checks.

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- [fullreview20260430 static-check-resource](../../doc/fullreview20260430/rust-compiler/static-check-resource.md)

## 対応内容

`ResourceDropElaborationPlan` を追加し、`ResourceCheckReport::functions[*].auto_drop_points` に記録された live drop fact だけを codegen-facing な drop elaboration 入力として構築するようにした。

この plan は `ResourceDropPlan` の candidate ではなく、initialized-state checker が EndScope 到達時点で実際に `Initialized` と確認した place のみを保持する。構築時には function/check の対応、drop point path の EndScope 解決、auto-drop place が対象 EndScope locals に含まれることを検証し、失敗は `ResourceDropElaborationPlanError` enum で分類する。

compiler pipeline では Resource IR cell gate 通過直後に `compute_resource_drop_elaboration_plan` を実行し、不整合があれば `resource.lower.incomplete` の hard error とする。これにより、次の HIR `passes::insert_drops` 削除作業で candidate plan や HIR scope walker を誤って authority に戻す危険を下げた。

## 回帰テスト

- `resource_ir_live_auto_drop_points_include_function_parameters`: non-Copy parameter の live drop fact が drop elaboration plan に出ることを確認。
- `resource_ir_drop_elaboration_plan_uses_checked_live_drop_facts`: move 済み outer local は plan に出ず、live inner shadow local だけが EndScope に解決されることを確認。
- `resource_ir_drop_elaboration_plan_rejects_invalid_checked_paths`: 壊れた path が `InvalidDropPointPath` enum error で拒否されることを確認。
- `resource_ir_drop_elaboration_plan_rejects_places_outside_end_scope`: EndScope locals に存在しない auto-drop place が `DropPlaceOutsideEndScope` enum error で拒否されることを確認。
- `nodesrc/test_resource_gate_order.js`: compiler が Resource IR cell gate 後に drop elaboration plan gate を実行することを監視。
- `nodesrc/test_resource_checker_responsibility.js`: Resource IR module 分割に `drop_elaboration.rs` を追加し、責務肥大を監視。
