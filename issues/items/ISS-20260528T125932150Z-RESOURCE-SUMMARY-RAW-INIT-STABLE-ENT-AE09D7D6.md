---
id: ISS-20260528T125932150Z-RESOURCE-SUMMARY-RAW-INIT-STABLE-ENT-AE09D7D6
title: "Resource summary raw-init stable entry rejects complete param facts"
area: core
status: open
resolved: false
priority: P1
type: performance
created: 2026-05-28
updated: 2026-05-28
target: "nepl-core/src/resource/resource_summary_value_cache/stable_mirror.rs; nepl-core/src/resource/initialized_summary_release_model.rs"
---

# ISS-20260528T125932150Z-RESOURCE-SUMMARY-RAW-INIT-STABLE-ENT-AE09D7D6: Resource summary raw-init stable entry rejects complete param facts

## 概要

After raw body dependency key support, RPN same-session code edit reports raw_init_param_facts_unstable_entry_bypasses=119. These summaries are complete param-fact leaves, but stable_raw_init_param_facts_leaf_entry still cannot mirror some contained type/projection/release requirement surfaces.

## 対象

- `nepl-core/src/resource/resource_summary_value_cache/stable_mirror.rs; nepl-core/src/resource/initialized_summary_release_model.rs`

## 根拠

- raw body dependency key checkpoint の RPN same-session code edit 測定で、dependency closure の `raw_init_param_facts_unstable_key_bypasses` は `176 -> 0` になった。
- 同じ測定で `raw_init_param_facts_unstable_entry_bypasses=119` が残ったため、dependency key ではなく stable value 変換境界が次の blocker である。
- `stable_raw_init_param_facts_leaf_entry` は complete param-fact leaf surface だけを受け取るが、内部の `RawCellInitializationParamCell` / `RawCellReleaseParamRequirement` の type、summary projection、release projection をすべて stable mirror 化できるわけではない。
- partial entry を保存すると replay 後の raw initialization proof を欠落させるため、失敗理由を分割してから fail-closed に再投影できる surface だけを追加する必要がある。

## 問題

After raw body dependency key support, RPN same-session code edit reports raw_init_param_facts_unstable_entry_bypasses=119. These summaries are complete param-fact leaves, but stable_raw_init_param_facts_leaf_entry still cannot mirror some contained type/projection/release requirement surfaces.

## 影響

The dependency closure key is now stable, but many otherwise eligible raw-init summaries still recompute on every code edit because their cache value cannot be stored.

## 修正方針

Split stable entry conversion failures by param cell type/projection and release requirement type/projection/kind, then add stable mirrors only for surfaces that can be replayed fail-closed with layout and type validation.

## 検証

RPN same-session code edit shows unstable_entry_bypasses decreasing, and focused stable_mirror tests prove each newly accepted release requirement or projection reprojects without dropping facts.
