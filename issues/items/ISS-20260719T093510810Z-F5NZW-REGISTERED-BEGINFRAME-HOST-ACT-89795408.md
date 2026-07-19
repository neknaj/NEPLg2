---
id: ISS-20260719T093510810Z-F5NZW-REGISTERED-BEGINFRAME-HOST-ACT-89795408
title: "F5nzw registered BeginFrame host action failure recovery"
area: gui-font
status: open
resolved: false
priority: P1
type: architecture
created: 2026-07-19
updated: 2026-07-19
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_host_action_failure_recovery.nepl
---

# ISS-20260719T093510810Z-F5NZW-REGISTERED-BEGINFRAME-HOST-ACT-89795408: F5nzw registered BeginFrame host action failure recovery

## 概要

F5nzl completion errors can only be aborted; registered continuation and lower retry/state recovery authority cannot be resumed together.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_host_action_failure_recovery.nepl`

## 根拠

- 未記入

## 問題

F5nzl completion errors can only be aborted; registered continuation and lower retry/state recovery authority cannot be resumed together.

## 影響

Unsupported or failed host action completion either discards recoverable pending/state authority or requires bypassing the typed F5nh/F5ng/F5nf/F5ne recovery graph.

## 修正方針

Consume the F5nzl error once and classify the complete lower graph into move-only retry-pending or recovered-loop-state owners while retaining the registered continuation and diagnostics.

## 検証

Actual unsupported SinkRejected retry recovery, source-policy full lower classification, normal compile isolation, lower regressions, release/trunk/CLI gates, and subagent reviews.
