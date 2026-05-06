---
id: ISS-20260506T111001704Z-RESOURCE-DROP-ELABORATION-PLAN-OMITS-C984305E
title: "Resource drop elaboration plan omits assignment overwrite drops"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/src/resource/drop_model.rs, nepl-core/src/resource/drop_plan_assignment.rs, nepl-core/src/resource/initialized_drop_assignment.rs, nepl-core/src/resource/drop_elaboration_validate.rs, nepl-core/src/resource/drop_point_resolve_assignment.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260506T111001704Z-RESOURCE-DROP-ELABORATION-PLAN-OMITS-C984305E: Resource drop elaboration plan omits assignment overwrite drops

## 概要

ResourceDropElaborationPlan currently records live EndScope auto drops only. HIR drop insertion still performs its own VarState walk to emit drop calls for overwriting initialized non-Copy bindings before Set/Assign. Removing passes::insert_drops without Resource IR assignment overwrite drop facts would miss required Drop calls, while keeping the HIR walker leaves a second authority.

## 対象

- `nepl-core/src/resource/initialized.rs, nepl-core/src/resource/drop_model.rs, nepl-core/src/resource/drop_elaboration.rs, nepl-core/tests/resource_ir.rs`

## 根拠

`doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 requires HIR `passes::insert_drops` to be replaced by checked Resource IR drop elaboration. Existing `ResourceDropElaborationPlan` represented only EndScope auto-drop facts, while old HIR drop insertion still had separate logic for dropping an initialized non-Copy local before `set` overwrites it.

## 問題

ResourceDropElaborationPlan currently records live EndScope auto drops only. HIR drop insertion still performs its own VarState walk to emit drop calls for overwriting initialized non-Copy bindings before Set/Assign. Removing passes::insert_drops without Resource IR assignment overwrite drop facts would miss required Drop calls, while keeping the HIR walker leaves a second authority.

## 影響

Stage 4 cannot safely replace HIR passes::insert_drops with checked Resource IR drop elaboration. Assignment overwrite cases such as tests/compiler/drop_overwrite.n.md would either leak/drop incorrectly after migration or force the old scope walker to remain as technical debt.

## 修正方針

Extend Resource IR live drop facts with a typed assignment-overwrite drop point kind. The initialized-state traversal should record a checked drop fact for an initialized non-Copy assignment target before the target is replaced, and ResourceDropElaborationPlan should validate and expose that fact separately from EndScope scope-local drops.

## 対応結果

`ResourceAutoDropKind` now distinguishes `ScopeLocal` from `AssignmentOverwrite`. Candidate drop planning and initialized-state traversal both emit assignment-overwrite drop points for initialized non-Copy assignment targets, and skip moved/uninitialized targets. The plan validator resolves these points through a typed `resolve_resource_drop_point_assignment` path and rejects mismatched assignment targets with a dedicated enum diagnostic case.

The HIR bridge now records `set` expression spans and target bindings, so assignment overwrite drop facts can be verified against source HIR without reviving the old scope walker. New helper modules keep the responsibilities split: assignment candidate creation, live initialized assignment recording, assignment path resolution, and drop-point-kind validation are separate from the main traversal modules.

関連 stage: [静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-への移行)

## 検証

Add focused Resource IR regressions proving assignment overwrite drop points are recorded only for initialized non-Copy targets and not for moved/uninitialized targets. Run cargo focused tests, resource source policy tests, issue checks, trunk build, and drop_overwrite nodesrc test.

確認済み:

- `cargo fmt --check`
- `cargo test -p nepl-core --test resource_ir resource_ir_drop_elaboration_plan -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_drop_elaboration_hir_bridge -- --nocapture`
- `cargo test -p nepl-core --test check_pipeline prepare_codegen_exposes_checked_resource_drop_elaboration_plan -- --nocapture`
- `cargo test -p nepl-core --test layout drop_insertion_uses_resource_drop_requirement_for_drop_classification -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_check_auto_drops_live_non_copy_local_at_scope_end -- --nocapture`
- `node nodesrc/test_resource_checker_responsibility.js`
- `node nodesrc/test_static_check_boundary_responsibility.js`
- `node nodesrc/test_resource_gate_order.js`
- `node nodesrc/issues.js check`
- `trunk build`
- `node nodesrc/tests.js -i tests/compiler/drop_overwrite.n.md -o tmp/drop-overwrite-assignment-drop-points.json --runner wasm --no-tree -j 1 --assert-io`: 1/1 passed
