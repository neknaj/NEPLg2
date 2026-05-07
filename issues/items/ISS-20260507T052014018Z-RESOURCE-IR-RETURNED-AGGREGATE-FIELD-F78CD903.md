---
id: ISS-20260507T052014018Z-RESOURCE-IR-RETURNED-AGGREGATE-FIELD-F78CD903
title: "Resource IR returned aggregate fields do not carry initialized raw range summaries"
area: core
status: open
resolved: false
priority: P2
type: architecture
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/resource/initialized_summary_byte_ranges.rs, nepl-core/src/resource/initialized_alias_flow_value_projection.rs, nepl-core/tests/kp.rs"
source: "doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-への移行"
---

# ISS-20260507T052014018Z-RESOURCE-IR-RETURNED-AGGREGATE-FIELD-F78CD903: Resource IR returned aggregate fields do not carry initialized raw range summaries

## 概要

Raw range summary collection can represent a raw header return, but aggregate field projection across a returned struct is still incomplete for address/count pairs. The fd_read bounded range fix exposed this because the previous unbounded payload fact had hidden the missing projection.

## 対象

- `nepl-core/src/resource/initialized_summary_byte_ranges.rs, nepl-core/src/resource/initialized_alias_flow_value_projection.rs, nepl-core/tests/kp.rs`

## 根拠

When local scanner returned a struct containing both a raw header pointer and its owned buffer pointer, caller-side guarded payload loads could not use the callee initialized range summary.

## 問題

Raw range summary collection can represent a raw header return, but aggregate field projection across a returned struct is still incomplete for address/count pairs. The fd_read bounded range fix exposed this because the previous unbounded payload fact had hidden the missing projection.

## 影響

Full scanner/self-host input structures that return metadata structs still need fixture reshaping or additional summary model support. This is a remaining parent issue for returned header / fd_read / capacity integration.

## 修正方針

Extend returned aggregate value projection summaries so address suffix and count suffix can be projected through struct fields without broadening raw memory initialization.

## 検証

Add a focused returned-aggregate scanner/header regression that passes without unknown-offset payload initialization.
