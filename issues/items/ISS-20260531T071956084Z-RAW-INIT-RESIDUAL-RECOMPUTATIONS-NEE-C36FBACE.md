---
id: ISS-20260531T071956084Z-RAW-INIT-RESIDUAL-RECOMPUTATIONS-NEE-C36FBACE
title: "raw init residual recomputations need function local invalidation"
area: core
status: open
resolved: false
priority: P1
type: performance
created: 2026-05-31
updated: 2026-05-31
target: "nepl-core/src/source_map.rs; nepl-core/src/resource/resource_summary_value_cache/context.rs; nepl-core/src/resource/resource_summary_value_cache/dependency_hash.rs; nepl-core/src/resource/resource_summary_value_cache/raw_init.rs"
---

# ISS-20260531T071956084Z-RAW-INIT-RESIDUAL-RECOMPUTATIONS-NEE-C36FBACE: raw init residual recomputations need function local invalidation

## 概要

RPN code edit still recomputes 81 raw-init summaries even though raw_init_param_facts hits 205 and bypasses are zero.

## 対象

- `nepl-core/src/source_map.rs; nepl-core/src/resource/resource_summary_value_cache/context.rs; nepl-core/src/resource/resource_summary_value_cache/dependency_hash.rs; nepl-core/src/resource/resource_summary_value_cache/raw_init.rs`

## 根拠

- `tmp/rpn_final_check_residual_type_fix_20260531.json` の edit delta では、`raw_init_param_facts_hits=205`、`raw_init_param_facts_bypasses=0` にもかかわらず `resource_raw_init_summary_recomputations=81` が残っている。
- subagent review では、`SourceMap::source_capability_policy_hash_for_file` が path、source hash、capability proof set を一体で hash し、dependency closure hash がその source policy hash を取り込むため、同一 file の小さな edit で広い raw-init invalidation が起きている可能性が高いと整理した。

## 問題

RPN code edit still recomputes 81 raw-init summaries even though raw_init_param_facts hits 205 and bypasses are zero.

## 影響

A tiny edit in examples/rpn.nepl likely invalidates too much source capability policy surface, so dependency closure keys cause safe but overly broad raw-init recomputation.

## 修正方針

Split source capability policy invalidation into function-local exact use-site surfaces and dependency body/type boundaries, preserving stale-proof rejection for capability byte-range changes while avoiding whole-file invalidation for unrelated body edits.

## 検証

Compare RPN same-session edits in the same function, another function in the same file, another imported file, and no stdlib change. Raw-init recomputation should only remain where function-local source policy or dependency body/type boundary actually changed.
