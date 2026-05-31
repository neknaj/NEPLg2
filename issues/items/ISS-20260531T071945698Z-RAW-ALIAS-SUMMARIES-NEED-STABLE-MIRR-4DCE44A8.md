---
id: ISS-20260531T071945698Z-RAW-ALIAS-SUMMARIES-NEED-STABLE-MIRR-4DCE44A8
title: "raw alias summaries need stable mirror and preseed cache"
area: core
status: open
resolved: false
priority: P1
type: performance
created: 2026-05-31
updated: 2026-05-31
target: "nepl-core/src/resource/initialized_alias_flow.rs; nepl-core/src/resource/resource_summary_value_cache"
---

# ISS-20260531T071945698Z-RAW-ALIAS-SUMMARIES-NEED-STABLE-MIRR-4DCE44A8: raw alias summaries need stable mirror and preseed cache

## 概要

RPN code edit still recomputes 288 raw alias summaries because raw alias fixed-point summaries have no stable mirror/preseed cache path.

## 対象

- `nepl-core/src/resource/initialized_alias_flow.rs; nepl-core/src/resource/resource_summary_value_cache`

## 根拠

- `tmp/rpn_final_check_residual_type_fix_20260531.json` の edit delta では、final check type bypass が 0 になった後も `resource_raw_alias_summary_recomputations=288` が残っている。
- subagent review では、`initialized_alias_flow.rs` が `SummaryWorklist::new(module)` で全到達関数を毎回 fixed-point に入れており、raw alias summary には stable cache / preseed path がないと確認した。

## 問題

RPN code edit still recomputes 288 raw alias summaries because raw alias fixed-point summaries have no stable mirror/preseed cache path.

## 影響

Resource IR incremental compile remains seconds-scale after raw-init, i32 scalar, and final check replay because raw alias summaries are rebuilt for every reached function and also feed downstream raw-init/i32/final-check dependencies.

## 修正方針

Design a stable raw alias summary mirror that stores only arena-independent alias facts, adds dependency/source-policy keys, and pre-seeds SummaryWorklist without storing TypeId, Span, SourceMap, or mutable graph state.

## 検証

RPN same-session code edit JSON should show raw_alias_summary_recomputations dropping substantially while raw_init_param_facts_bypasses remains zero and existing Resource IR safety tests pass.
