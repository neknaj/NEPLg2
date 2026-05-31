---
id: ISS-20260531T012124326Z-RESOURCE-SUMMARY-RAW-INIT-VALUE-REPR-88633148
title: "Resource summary raw-init value reprojection needs projection canonicalization"
area: core
status: open
resolved: false
priority: P1
type: performance
created: 2026-05-31
updated: 2026-05-31
target: "nepl-core/src/resource/resource_summary_value_cache/stable_mirror.rs; nepl-core/src/resource/initialized_summary_build_value_cache.rs"
---

# ISS-20260531T012124326Z-RESOURCE-SUMMARY-RAW-INIT-VALUE-REPR-88633148: Resource summary raw-init value reprojection needs projection canonicalization

## 概要

After instantiated generic nominal mapping and signature duplicate reprojection fixes, RPN same-session code edit still reports raw_init_param_facts_reprojection_value_bypasses=25 on first compile and 50 cumulatively after the second compile. Context construction is now 0, so the residual failure is inside stable raw-init value-to-summary replay.

## 対象

- `nepl-core/src/resource/resource_summary_value_cache/stable_mirror.rs; nepl-core/src/resource/initialized_summary_build_value_cache.rs`

## 根拠

- `ISS-20260528T123956303Z-RESOURCE-SUMMARY-TYPE-REPROJECTION-N-78929A8E` で `ResourceSummaryTypeReprojection` の context 構築失敗は 0 まで減った。
- 同 checkpoint の RPN same-session code edit 測定では、初回 compile の `raw_init_param_facts_reprojection_bypasses=25` はすべて `raw_init_param_facts_reprojection_value_bypasses=25` だった。
- 2 回目 compile では `raw_init_param_facts_hits=142` / `resource_summary_value_replay_hits=142` まで伸びたが、累積 `raw_init_param_facts_reprojection_value_bypasses=50` が残った。
- `raw_init_param_facts_unstable_entry_bypasses=0` と `raw_init_param_facts_unstable_key_bypasses=0` は維持されているため、残件は stable entry 生成や key 生成ではなく、entry から現在 compile の summary surface へ戻す value replay 境界にある。

## 問題

After instantiated generic nominal mapping and signature duplicate reprojection fixes, RPN same-session code edit still reports raw_init_param_facts_reprojection_value_bypasses=25 on first compile and 50 cumulatively after the second compile. Context construction is now 0, so the residual failure is inside stable raw-init value-to-summary replay.

## 影響

Raw-init param facts store/hit coverage improved substantially, but remaining value reprojection misses still force recomputation for some stdlib functions and keep code-edit compile time around 8.5 seconds.

## 修正方針

Split reproject_raw_init_param_facts_leaf_entry value failures by param cell projection/type and release requirement projection/type, then canonicalize the replayable projection/type cases without weakening open generic ambiguity checks.

## 検証

RPN same-session code edit should keep reprojection_context_bypasses=0 and decrease reprojection_value_bypasses below the current first_compile value of 25 while preserving resource_summary_value_raw_init_param_facts_unstable_entry_bypasses=0.
