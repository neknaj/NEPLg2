---
id: ISS-20260506T090109381Z-RESOURCE-DROP-ELABORATION-PLAN-LACKS-82B39C85
title: "Resource drop elaboration plan lacks source binding names"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/src/resource/drop_elaboration.rs, nepl-core/src/resource/model.rs, nepl-core/src/resource/lower.rs"
---

# ISS-20260506T090109381Z-RESOURCE-DROP-ELABORATION-PLAN-LACKS-82B39C85: Resource drop elaboration plan lacks source binding names

## 概要

ResourceDropElaborationPlan carries checked Resource IR places, but shadowed locals can be renamed internally (for example x#0) while HIR/backend drop call generation must refer to the source binding visible at that insertion point. The plan therefore is not yet a complete input for replacing HIR passes::insert_drops.

## 対象

- `nepl-core/src/resource/drop_elaboration.rs, nepl-core/src/resource/model.rs, nepl-core/src/resource/lower.rs`

## 根拠

- 未記入

## 問題

ResourceDropElaborationPlan carries checked Resource IR places, but shadowed locals can be renamed internally (for example x#0) while HIR/backend drop call generation must refer to the source binding visible at that insertion point. The plan therefore is not yet a complete input for replacing HIR passes::insert_drops.

## 影響

If codegen consumes only Resource IR place names, shadowed local drops can target non-existent backend names, or the implementation may fall back to HIR scope traversal to recover source names, preserving the second drop authority.

## 修正方針

Record source binding names in Resource IR local declarations and produce ResourceDropElaborationDrop entries that pair checked places with their source binding names. Validate missing bindings with a typed enum error and keep the compiler gate authoritative.

## 検証

Focused Resource IR tests should prove that shadowed internal places keep the original source name in the drop elaboration plan, parameter drops keep parameter names, and missing bindings are rejected by enum errors. Source policy and issue checks must pass.

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- [fullreview20260430 static-check-resource](../../doc/fullreview20260430/rust-compiler/static-check-resource.md)

## 対応内容

Resource IR の `DeclareLocal` に `source_name` を追加し、shadowing のために `PlaceRoot::Local("x#0")` のような内部 place 名へ分離した場合でも、元の source binding 名 `x` を保持するようにした。

`ResourceDropElaborationPlan` は `ResourceAutoDrop` を直接公開するのではなく、`ResourceDropElaborationDrop` と `ResourceDropElaborationPoint` を持つ構造に変更した。各 drop entry は checked `place`、backend/HIR が参照すべき `source_name`、`ResourceDropRequirement`、span を保持する。

`drop_elaboration_bindings.rs` は function parameter、`DeclareLocal`、match arm binding から place/source-name 対応を収集する。対応が存在しない live drop fact は `MissingDropBinding` enum error として compiler gate で `resource.lower.incomplete` へ写像する。

## 回帰テスト

- `resource_ir_live_auto_drop_points_include_function_parameters`: parameter drop entry の `source_name` が `_g` になることを確認。
- `resource_ir_drop_elaboration_plan_uses_checked_live_drop_facts`: shadowed internal place `x#...` が source binding 名 `x` を保持し、move 済み outer `x` は plan に出ないことを確認。
- `resource_ir_drop_elaboration_plan_rejects_missing_source_binding`: EndScope locals に存在しても source binding 名が解決できない place を `MissingDropBinding` enum error で拒否することを確認。
- `nodesrc/test_resource_checker_responsibility.js`: binding metadata 収集 module と line limit を監視。
