---
id: ISS-20260528T125932150Z-RESOURCE-SUMMARY-RAW-INIT-STABLE-ENT-AE09D7D6
title: "Resource summary raw-init stable entry rejects complete param facts"
area: core
status: verified
resolved: true
priority: P1
type: performance
created: 2026-05-28
updated: 2026-05-31
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

## 解決

2026-05-31 の checkpoint で、raw-init param-facts stable entry の release requirement projection を Resource summary stable mirror に載せた。

- `ResourceOffset::{Symbolic, ScaledSymbolic, Offset, ScaledOffset}` の operand が関数 parameter relative に再投影できる場合は、stable offset として保持する。
- operand が callee-local で stable identity を持たない場合は、stale な local place を保存せず、既存 overlap semantics と同じ conservative な `Unknown` offset へ正規化する。
- `ResourceOffset::Unknown` / `SummaryOffset::Unknown` も stable mirror value として保存し、現在 compile の Resource IR context へ `Unknown` として再投影する。
- release requirement の type は引き続き `ResourceSummaryStableTypeKey` で検証し、type key が安定しない場合は fail-closed に no-store へ戻す。

RPN same-session code edit の release Web 測定では、`raw_init_param_facts_unstable_entry_bypasses` が `119 -> 0` になった。初回 `raw_init_param_facts_stores=23`、2 回目 `raw_init_param_facts_hits=23` / `resource_summary_value_replay_hits=23` を確認した。

残件は `raw_init_param_facts_reprojection_bypasses=165` と `raw_init_param_facts_incomplete_leaf_bypasses=37` へ移ったため、前者は `ISS-20260528T123956303Z-RESOURCE-SUMMARY-TYPE-REPROJECTION-N-78929A8E`、後者は `ISS-20260528T123956163Z-RESOURCE-SUMMARY-RAW-INIT-CACHE-NEED-245DC1A5` で継続する。
