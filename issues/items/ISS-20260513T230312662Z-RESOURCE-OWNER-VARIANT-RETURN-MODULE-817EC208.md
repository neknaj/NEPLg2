---
id: ISS-20260513T230312662Z-RESOURCE-OWNER-VARIANT-RETURN-MODULE-817EC208
title: "Resource owner variant return module exceeds responsibility split limit after Vec owner fix"
area: core
status: fixed
resolved: true
priority: P1
type: maintenance
created: 2026-05-13
updated: 2026-05-13
target: "nepl-core/src/resource/owner_summary_variant_return.rs, nepl-core/src/resource/owner_summary_variant_return_sources.rs, nepl-core/src/resource/owner_summary_variant_paths.rs, nepl-core/src/resource/owner_variant_utils.rs, nepl-core/src/resource/owner_variant_condition_truth.rs, nepl-core/src/resource/owner_variant_record.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260513T230312662Z-RESOURCE-OWNER-VARIANT-RETURN-MODULE-817EC208: Resource owner variant return module exceeds responsibility split limit after Vec owner fix

## 概要

Source policy reports owner_summary_variant_return.rs has 301 lines while the enforced responsibility split limit is 280. The module now mixes returned owner source collection from OwnerTable/raw aliases with variant payload return materialization and owner conflict merging.

## 対象

- `nepl-core/src/resource/owner_summary_variant_return.rs, nepl-core/src/resource/owner_summary_variant_return_sources.rs, nepl-core/src/resource/owner_summary_variant_paths.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `node nodesrc/test_resource_checker_responsibility.js` が `owner_summary_variant_return.rs has 301 lines; responsibility split limit is 280` を報告した。
- `owner_summary_variant_return.rs` は returned value から `OwnerProjectionReturnSummary` / `OwnerProjectionSource` を集める owner table / raw alias traversal と、variant payload return の materialization / owner conflict merge を同じ module に持っていた。
- 分割後の同 policy は次の隠れた超過として `owner_variant_utils.rs has 240 lines; responsibility split limit is 220` を報告した。ここには projection source lookup、unique push helper、value-condition truth evaluation、payload suffix binding が混在していた。

## 問題

Source policy reports owner_summary_variant_return.rs has 301 lines while the enforced responsibility split limit is 280. The module now mixes returned owner source collection from OwnerTable/raw aliases with variant payload return materialization and owner conflict merging. After that split, the same policy exposes owner_variant_utils.rs as the next hidden responsibility violation because condition truth evaluation is mixed into general variant helper utilities.

## 影響

Resource IR owner-summary code is becoming harder to audit around enum payload owner returns. Future safety fixes can re-concentrate source collection and variant return merging, increasing the risk of stale owner/provenance behavior.

## 修正方針

Split returned owner source collection into a dedicated module, keep variant projection materialization in owner_summary_variant_return.rs, move value-condition truth evaluation into a dedicated module, and monitor all modules through the resource responsibility policy.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js, cargo check -p nepl-core --tests, node nodesrc/issues.js check --dir issues, and git diff --check.

## 解決内容

- `owner_summary_variant_return_sources.rs` を追加し、return value / descendant / alias descendant から owner return source を集める処理を移した。
- `owner_summary_variant_return.rs` は variant payload projection return の materialization と同一 target owner の merge policy に集中する形へ戻した。
- `owner_variant_condition_truth.rs` を追加し、`OwnerValueCondition` を実引数と raw alias table から評価する処理を `owner_variant_utils.rs` から分離した。
- `nodesrc/test_resource_checker_responsibility.js` に新 module の監視と line limit を追加し、同じ責務再集中が再発した場合に検出されるようにした。
