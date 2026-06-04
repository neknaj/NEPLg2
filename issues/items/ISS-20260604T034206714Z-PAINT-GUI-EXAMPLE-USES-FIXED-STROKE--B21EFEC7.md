---
id: ISS-20260604T034206714Z-PAINT-GUI-EXAMPLE-USES-FIXED-STROKE--B21EFEC7
title: "Paint GUI example uses fixed stroke slots and sentinel values instead of typed stroke storage"
area: examples
status: open
resolved: false
priority: P1
type: architecture
created: 2026-06-04
updated: 2026-06-04
target: examples/gui_paint.nepl
---

# ISS-20260604T034206714Z-PAINT-GUI-EXAMPLE-USES-FIXED-STROKE--B21EFEC7: Paint GUI example uses fixed stroke slots and sentinel values instead of typed stroke storage

## 概要

Subagent audit found gui_paint.nepl using three stroke slots and a 255 sentinel for missing or inactive values. This conflicts with the Zenn Option/Result/enum guidance and leaves the example closer to pointer-event smoke testing than a paint application.

## 対象

- `examples/gui_paint.nepl`

## 根拠

- 未記入

## 問題

Subagent audit found gui_paint.nepl using three stroke slots and a 255 sentinel for missing or inactive values. This conflicts with the Zenn Option/Result/enum guidance and leaves the example closer to pointer-event smoke testing than a paint application.

## 影響

Multiple strokes, clear/history, capacity overflow, and pointer cancellation cannot be modeled cleanly, and sentinel state can hide invalid interactions.

## 修正方針

Represent strokes with Option or bounded collection storage, remove sentinel values from the model, and return Result on capacity overflow or invalid pointer state.

## 検証

Add regular tests for pointer down/move/up, multiple stroke points, clear, color change, capacity overflow, and cancellation.
