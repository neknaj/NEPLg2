---
id: ISS-20260507T174020115Z-SOURCE-CAPABILITIES-KEEP-RAW-MEMORY--41F29FB6
title: "Source capabilities keep raw memory boundary as ad hoc boolean"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/source_map.rs, nepl-core/src/loader.rs, nepl-core/tests/effects.rs"
---

# ISS-20260507T174020115Z-SOURCE-CAPABILITIES-KEEP-RAW-MEMORY--41F29FB6: Source capabilities keep raw memory boundary as ad hoc boolean

## 概要

SourceCapabilities stores the compiler-owned raw-memory-boundary privilege as a dedicated boolean. As Stage 5/6 adds more compiler capabilities, boolean fields make privilege classification ad hoc instead of enum-first.

## 対象

- `nepl-core/src/source_map.rs, nepl-core/src/loader.rs, nepl-core/tests/effects.rs`

## 根拠

- 未記入

## 問題

SourceCapabilities stores the compiler-owned raw-memory-boundary privilege as a dedicated boolean. As Stage 5/6 adds more compiler capabilities, boolean fields make privilege classification ad hoc instead of enum-first.

## 影響

Raw memory boundary checks can grow by adding flags rather than typed capability variants, weakening exhaustiveness and making static-check privilege audits harder.

## 修正方針

Introduce a SourceCapability enum and store capabilities as an enum-keyed set. Keep raw-memory-boundary behavior unchanged while making future source privileges explicit typed variants.

## 検証

Run cargo fmt/check focused source capability and loader tests, trunk build if Rust output changes, focused memory_safety doctests, node nodesrc/issues.js check, and source policy regressions.
