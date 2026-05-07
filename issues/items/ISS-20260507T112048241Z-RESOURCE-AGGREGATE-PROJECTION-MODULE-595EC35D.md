---
id: ISS-20260507T112048241Z-RESOURCE-AGGREGATE-PROJECTION-MODULE-595EC35D
title: "Resource aggregate projection module exceeds responsibility split limit after tuple selector lowering"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/resource/lower_aggregate_projection.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260507T112048241Z-RESOURCE-AGGREGATE-PROJECTION-MODULE-595EC35D: Resource aggregate projection module exceeds responsibility split limit after tuple selector lowering

## 概要

After syncing remote main cc5d7662, node nodesrc/run_source_policy_regressions.js --warn-only reports lower_aggregate_projection.rs has 204 lines while the enforced responsibility split limit is 180. The tuple index selector fix added numeric selector lowering into the aggregate projection module, so struct field projection, tuple field projection, selector parsing, and coverage/lowering helpers are again concentrated in one file.

## 対象

- `nepl-core/src/resource/lower_aggregate_projection.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `9562d4c7` の stdlib/string search 分割後、remote main `cc5d7662` を含む main 上で `trunk build` と focused stdlib tests は通過した。
- その後の `node nodesrc/run_source_policy_regressions.js --warn-only` で `nodesrc/test_resource_checker_responsibility.js` が `lower_aggregate_projection.rs has 204 lines; responsibility split limit is 180` を報告した。
- stdlib/string/search 関連の policy は通過しており、この warning は `cc5d7662` の tuple selector lowering によって Resource IR aggregate projection module が再肥大化した別問題である。

## 問題

After syncing remote main cc5d7662, node nodesrc/run_source_policy_regressions.js --warn-only reports lower_aggregate_projection.rs has 204 lines while the enforced responsibility split limit is 180. The tuple index selector fix added numeric selector lowering into the aggregate projection module, so struct field projection, tuple field projection, selector parsing, and coverage/lowering helpers are again concentrated in one file.

## 影響

Resource IR aggregate projection lowering is part of memory-safety and type-safety static checking. Letting the module grow past its responsibility guard makes selector semantics harder to audit and weakens the source-policy signal that should keep ResourceIR lowering reviewable.

## 修正方針

Split lower_aggregate_projection.rs again by selector responsibilities, for example separating AggregateFieldSelector parsing/classification from PlaceProjection construction and coverage matching, without weakening tuple/struct projection semantics.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js and node nodesrc/run_source_policy_regressions.js --warn-only; both must pass without the lower_aggregate_projection.rs line-limit warning.
