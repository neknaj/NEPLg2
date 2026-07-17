---
id: ISS-20260717T153830514Z-F5NZE-REGISTERED-BEGINFRAME-VIRTUAL--8902C86C
title: "F5nze registered BeginFrame virtual drain connection"
area: gui-font
status: open
resolved: false
priority: P1
type: architecture
created: 2026-07-17
updated: 2026-07-17
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_virtual_drain.nepl
---

# ISS-20260717T153830514Z-F5NZE-REGISTERED-BEGINFRAME-VIRTUAL--8902C86C: F5nze registered BeginFrame virtual drain connection

## 概要

The registered actual BeginFrame record authority stops before the existing F5mv virtual drain.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_virtual_drain.nepl`

## 根拠

- F5nzdはactual registered pipelineのBeginFrame command stepとF5mu typed projectionを同じmove-only authorityに保持する。
- F5mvは`GuiRgba8888CompositorTileRlePresentHostCommandRecord`だけを受け取る既存のtarget-free stream validation authorityである。

## 問題

The registered actual BeginFrame record authority stops before the existing F5mv virtual drain.

## 影響

The registered stroke compositor cannot validate its actual typed BeginFrame record with the standard compositor drain authority.

## 修正方針

Consume the F5nzd owner losslessly, pass its actual Record(BeginFrame) to F5mv exactly once, retain/recover the command step across success and typed failure, and stop before host execution.

## 検証

Focused runtime fixture, F5mv and F5nzd regressions, source policy, normal compile isolation, trunk build, CLI JSON, and subagent reviews.
