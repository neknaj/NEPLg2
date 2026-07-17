---
id: ISS-20260717T145826052Z-F5NZD-REGISTERED-BEGINFRAME-HOST-COM-B047CB89
title: "F5nzd registered BeginFrame host-command record projection"
area: gui-font
status: open
resolved: false
priority: P1
type: architecture
created: 2026-07-17
updated: 2026-07-17
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_record.nepl
---

# ISS-20260717T145826052Z-F5NZD-REGISTERED-BEGINFRAME-HOST-COM-B047CB89: F5nzd registered BeginFrame host-command record projection

## 概要

The registered BeginFrame command step is not retained with its existing F5mu typed record projection.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_record.nepl`

## 根拠

- 未記入

## 問題

The registered BeginFrame command step is not retained with its existing F5mu typed record projection.

## 影響

The registered stroke compositor path cannot expose an actual typed BeginFrame record authority before virtual drain.

## 修正方針

Borrow the opaque F5nyy BeginFrame step into F5mu exactly once and retain the move-only step plus Copy BeginFrame record result without entering F5mv or host execution.

## 検証

Focused runtime fixture, source policy, normal compile isolation, lower regressions, trunk build, CLI JSON, and subagent reviews.
