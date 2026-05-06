---
id: ISS-20260506T030752155Z-RESOURCE-HIR-PROJECTION-COVERAGE-MOD-ED65CFB3
title: "Resource HIR projection coverage module exceeds responsibility split limit"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/src/resource/coverage_hir_projection.rs, nepl-core/src/resource/coverage_hir_projection_aggregate.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260506T030752155Z-RESOURCE-HIR-PROJECTION-COVERAGE-MOD-ED65CFB3: Resource HIR projection coverage module exceeds responsibility split limit

## 概要

After syncing remote main at 412a550a, source policy reports coverage_hir_projection.rs has 296 lines while the responsibility split limit is 280. The newly split projection coverage module already exceeds its guard.

## 対象

- `nepl-core/src/resource/coverage_hir_projection.rs, nepl-core/src/resource/coverage_hir_projection_aggregate.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `node nodesrc/run_source_policy_regressions.js --warn-only` が `nodesrc/test_resource_checker_responsibility.js` で `coverage_hir_projection.rs has 296 lines; responsibility split limit is 280` を報告した。
- この warning は `ISS-20260425T000000Z-RV-STDLIB-009` の nm HTML inline split 検証中に、remote main `412a550a` 取り込み後の別件として検出した。

## 問題

After syncing remote main at 412a550a, source policy reports coverage_hir_projection.rs has 296 lines while the responsibility split limit is 280. The newly split projection coverage module already exceeds its guard.

## 影響

Leaving field/reference/compiler projection coverage classification over the split limit lets Resource IR coverage responsibilities reconcentrate immediately after the coverage_hir split, making static/resource checker changes harder to audit.

## 修正方針

Split coverage_hir_projection.rs by responsibility, for example separating expression projection classification from compiler lowered projection classification, without weakening Resource IR coverage checks.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js and node nodesrc/run_source_policy_regressions.js --warn-only; the coverage_hir_projection.rs warning must disappear.

## 2026-05-06 対応結果

`coverage_hir_projection.rs` から aggregate field matching の責務を `coverage_hir_projection_aggregate.rs` へ分離した。

- `coverage_hir_projection.rs`: `get` / `get_field` / `get_field_ref` / compiler `load(add ...)` / reference-address projection の entrypoint 判定を担当する。
- `coverage_hir_projection_aggregate.rs`: field name / selector / offset から aggregate field が expected type に対応するかの分類を担当する。
- `nodesrc/test_resource_checker_responsibility.js`: 新 module の存在、`resource/mod.rs` の module declaration、行数上限を固定した。

検証:

- `cargo fmt -p nepl-core`: passed
- `cargo test -p nepl-core resource_ir --tests`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: `coverage_hir_projection.rs` 超過は解消。次の別件として `lower_aggregate.rs has 366 lines; responsibility split limit is 320` を検出した。
- `node nodesrc/run_source_policy_regressions.js --warn-only`: coverage_hir_projection 関連は passed。既知化予定の別件 `lower_aggregate.rs has 366 lines; responsibility split limit is 320` warning は継続。
