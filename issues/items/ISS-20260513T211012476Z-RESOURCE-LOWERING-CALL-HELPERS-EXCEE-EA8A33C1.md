---
id: ISS-20260513T211012476Z-RESOURCE-LOWERING-CALL-HELPERS-EXCEE-EA8A33C1
title: "Resource lowering and initialized drop helpers exceed responsibility boundaries"
area: resource
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-05-13
updated: 2026-05-13
target: "nepl-core/src/resource/lower.rs; nepl-core/src/resource/lower_call.rs; nepl-core/src/resource/initialized_drop_scope.rs; nepl-core/src/resource/initialized_drop_requirement.rs; nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260513T211012476Z-RESOURCE-LOWERING-CALL-HELPERS-EXCEE-EA8A33C1: Resource lowering and initialized drop helpers exceed responsibility boundaries

## 概要

The aggregate source-policy runner first reported lower.rs has 1164 lines while the responsibility split limit is 1150. After that split, the same policy exposed initialized_drop_scope.rs at 215 lines with an 80-line limit. The excess is not just line count: call-effect classification, ResourceCallTarget lowering, function-value effect conversion, generic FuncRef base-name helper, and partial structural drop-requirement calculation live in broader modules than their responsibilities require.

## 対象

- `nepl-core/src/resource/lower.rs; nepl-core/src/resource/lower_call.rs; nepl-core/src/resource/initialized_drop_scope.rs; nepl-core/src/resource/initialized_drop_requirement.rs; nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `node nodesrc/run_source_policy_regressions.js` failed at `lower.rs has 1164 lines; responsibility split limit is 1150`.
- After extracting call helpers, `node nodesrc/test_resource_checker_responsibility.js` then exposed `initialized_drop_scope.rs has 215 lines; responsibility split limit is 80`.
- `lower.rs` owned call effect classification and target lowering even though these are shared by raw-address and aggregate lowering.
- `initialized_drop_scope.rs` mixed live scope-drop state mutation with recursive partial structural drop-requirement calculation.

## 問題

The aggregate source-policy runner reports Resource lowering responsibility drift. `lower.rs` exceeded its line limit because call-effect / call-target helpers lived in the root lowering module. `initialized_drop_scope.rs` exceeded its line limit because it combined scope cleanup and recursive partial drop-requirement construction.

## 影響

Resource IR lowering and initialized-state drop handling can drift back into central modules, making effect/resource call authority and drop-obligation proof harder to review. This weakens the source-policy guard used for memory-safety work.

## 修正方針

Extract call-target/effect/base-name lowering into a dedicated `resource/lower_call.rs` module, reuse it from `lower.rs` and other `lower_*` modules. Extract partial structural drop-requirement proof into `resource/initialized_drop_requirement.rs`, leaving `initialized_drop_scope.rs` focused on scope-local state cleanup and auto-drop recording. Update responsibility policy limits so both responsibilities stay monitored.

## 検証

- `cargo fmt --package nepl-core --check`: passed
- `cargo check -p nepl-core --tests`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `node nodesrc/run_source_policy_regressions.js`: passed
- `node nodesrc/test_abstraction_static_verification_policy.js`: passed
- `node nodesrc/issues.js check --dir issues`: passed

## 対応内容

- `lower_call.rs` を追加し、call effect、intrinsic effect、function value effect、`ResourceCallTarget` lowering、`FuncRef` base-name helper を集約した。
- `lower.rs` / `lower_aggregate.rs` / `lower_condition.rs` / `lower_raw_address.rs` / `lower_raw_address_return.rs` は call helper を `lower_call.rs` から参照するようにした。
- `initialized_drop_requirement.rs` を追加し、partial structural drop-requirement proof を `initialized_drop_scope.rs` から分離した。
- `initialized_drop_scope.rs` は EndScope の live cell state cleanup、alias cleanup、auto-drop record 生成に集中する形へ戻した。
- `nodesrc/test_resource_checker_responsibility.js` に新 module の存在、`mod` 宣言、line limit、責務移動の source policy を追加した。
