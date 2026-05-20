---
id: ISS-20260520T165035652Z-RAW-FILL-LIFECYCLE-EVENT-CONFLATES-C-3A87C0FB
title: "Raw fill lifecycle event conflates Copy element proof with destructive discard"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-20
updated: 2026-05-20
target: "nepl-core/src/resource/raw_cell_lifecycle.rs, nepl-core/src/resource/initialized_raw_fill.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260520T165035652Z-RAW-FILL-LIFECYCLE-EVENT-CONFLATES-C-3A87C0FB: Raw fill lifecycle event conflates Copy element proof with destructive discard

## 概要

RawCellLifecycleEvent::FillCopyElements can be constructed for non-Copy payloads and then silently avoids creating initialized range evidence. This keeps current safety but represents a failed Copy proof as a successful event with no postcondition instead of making Copy evidence part of the event type.

## 対象

- `nepl-core/src/resource/raw_cell_lifecycle.rs, nepl-core/src/resource/initialized_raw_fill.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- 未記入

## 問題

RawCellLifecycleEvent::FillCopyElements can be constructed for non-Copy payloads and then silently avoids creating initialized range evidence. This keeps current safety but represents a failed Copy proof as a successful event with no postcondition instead of making Copy evidence part of the event type.

## 影響

The checker program itself is harder to audit: callers can request a Copy-element fill without carrying a typed Copy proof, and correctness depends on a hidden branch inside the lifecycle handler. That weakens enum/match based static verification and can hide future non-Copy slot lifecycle bugs.

## 修正方針

Split the lifecycle variants so Copy-element fill requires typed Copy evidence before construction, while non-Copy fill/destructive overwrite is represented as a separate discard-only event or a diagnostic path.

## 検証

Add source-policy and Resource IR regressions proving non-Copy fill cannot enter the Copy-element lifecycle variant, while Copy fill still creates initialized range evidence.
