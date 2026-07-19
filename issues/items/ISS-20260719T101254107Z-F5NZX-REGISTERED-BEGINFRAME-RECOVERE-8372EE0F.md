---
id: ISS-20260719T101254107Z-F5NZX-REGISTERED-BEGINFRAME-RECOVERE-8372EE0F
title: "F5nzx registered BeginFrame recovered-state scheduler decision"
area: gui-font
status: open
resolved: false
priority: P1
type: architecture
created: 2026-07-19
updated: 2026-07-19
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_recovered_state_scheduler_decision.nepl
---

# ISS-20260719T101254107Z-F5NZX-REGISTERED-BEGINFRAME-RECOVERE-8372EE0F: F5nzx registered BeginFrame recovered-state scheduler decision

## 概要

F5nzw recovered loop state has no typed scheduler decision boundary, so retry resubmission cannot define finite retry, yield, or abort authority safely.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_recovered_state_scheduler_decision.nepl`

## 根拠

- 未記入

## 問題

F5nzw recovered loop state has no typed scheduler decision boundary, so retry resubmission cannot define finite retry, yield, or abort authority safely.

## 影響

Blind RetryPending resubmission can repeat unsupported execution indefinitely, while recovered loop state cannot be resumed or aborted with its registered continuation and diagnostics intact.

## 修正方針

Consume only F5nzw RecoveredState authority and classify a caller-supplied value-only decision into resume-ready or abort-ready move-only handoffs, preserving category and exact diagnostic without executing schedulers or host work.

## 検証

Actual DriverCompletionFailed recovery and resume/abort fixtures, source-policy isolation, normal compile, lower regressions, release/trunk/CLI gates, and subagent reviews.
