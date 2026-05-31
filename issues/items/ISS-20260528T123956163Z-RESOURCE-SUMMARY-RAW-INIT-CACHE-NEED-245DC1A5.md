---
id: ISS-20260528T123956163Z-RESOURCE-SUMMARY-RAW-INIT-CACHE-NEED-245DC1A5
title: "Resource summary raw-init cache needs complete byte-range and variant leaf mirrors"
area: core
status: verified
resolved: true
priority: P1
type: performance
created: 2026-05-28
updated: 2026-05-31
target: "nepl-core/src/resource/resource_summary_value_cache/stable_mirror.rs; nepl-core/src/resource/initialized_summary*.rs"
---

# ISS-20260528T123956163Z-RESOURCE-SUMMARY-RAW-INIT-CACHE-NEED-245DC1A5: Resource summary raw-init cache needs complete byte-range and variant leaf mirrors

## 概要

RPN same-session code edit reports raw_init_param_facts_incomplete_leaf_bypasses=37. The current raw-init stable mirror only stores param_cells and param_release_requirements, so summaries containing byte-range, return, or variant facts are correctly rejected.

## 対象

- `nepl-core/src/resource/resource_summary_value_cache/stable_mirror.rs; nepl-core/src/resource/initialized_summary*.rs`

## 根拠

- `ISS-20260528T110220373Z-RESOURCE-SUMMARY-CACHE-NEEDS-QUALIFI-08D1AA04` の verified 測定で、`raw_init_param_facts_incomplete_leaf_bypasses=37` が残った。
- 現在の `ResourceSummaryStableRawInitParamFactsLeafEntry` は `param_cells` と `param_release_requirements` だけを保存し、`return_cells` / byte-range facts / variant facts を含む summary surface は完全再投影できないため no-store に倒している。
- partial summary を保存すると replay 後の raw initialization proof が欠落するため、byte-range / variant / return facts は個別にではなく complete entry として設計する必要がある。
- 2026-05-31 の raw-init stable entry checkpoint 後も RPN same-session code edit の初回 compile で `raw_init_param_facts_incomplete_leaf_bypasses=37` が残った。これは complete param facts stable entry の問題ではなく、byte-range / variant / return facts の complete mirror 不足として継続する。
- 2026-05-31 の type reprojection checkpoint 後も `raw_init_param_facts_incomplete_leaf_bypasses=37` は変わっていない。generic nominal / signature duplicate 再投影は store/hit を増やしたが、return / byte-range / variant facts を含む summary surface は依然として complete mirror 設計待ちである。

## 問題

RPN same-session code edit reports raw_init_param_facts_incomplete_leaf_bypasses=37. The current raw-init stable mirror only stores param_cells and param_release_requirements, so summaries containing byte-range, return, or variant facts are correctly rejected.

## 影響

A significant part of raw initialization proof work remains outside the session cache, especially string and collection helpers that track initialized byte ranges or variant-dependent payload facts.

## 修正方針

Design stable mirrors for byte-range and variant raw-init summary surfaces as complete entries, with fail-closed replay and layout/type revalidation. Do not store partial summaries that would hide missing facts.

## 完了内容

2026-05-31 の complete raw-init leaf checkpoint で、`RawCellInitializationFunctionSummary` の `return_cells`、`return_byte_ranges`、`param_cells`、`param_byte_ranges`、`param_release_requirements`、`variant_param_cells`、`variant_param_byte_ranges`、`variant_required_param_cells`、`variant_conditions` を同じ stable entry に保存し、fail-closed に再投影するようにした。

partial summary は保存せず、entry は complete leaf surface 全体を復元できる場合だけ store される。

## 検証

- focused regression で return cell / byte-range / variant / release requirement を含む complete summary が preseed 後に同じ summary surface として replay されることを確認した。
- corrupted return byte-range layout と non-final raw `Deref` fallback が fail-closed になる regression を追加した。
- RPN same-session code edit の compiled-output miss 測定で、base `compile_ms=8677`、edit `compile_ms=6586`、edit delta は `raw_init_param_facts_hits=205`、`resource_summary_value_replayed_ops=238`、`raw_init_param_facts_incomplete_leaf_bypasses=0` だった。

## 継続課題

complete mirror によって `incomplete_leaf` の根本原因は解消した。一方で、同測定では edit delta として `raw_init_param_facts_reprojection_value_bypasses=15`、`param_cell_result_type=15` が残る。これは partial mirror ではなく type canonicalization の別問題であるため、`ISS-20260531T132755602Z-RAW-INIT-COMPLETE-LEAF-REPROJECTION-TYPE-CANON-4E8A1A2C` に分離する。
