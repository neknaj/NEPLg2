---
id: ISS-20260719T101254107Z-F5NZX-REGISTERED-BEGINFRAME-RECOVERE-8372EE0F
title: "F5nzx registered BeginFrame recovered-state scheduler decision"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-19
updated: 2026-07-19
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_recovered_state_scheduler_decision.nepl
---

# ISS-20260719T101254107Z-F5NZX-REGISTERED-BEGINFRAME-RECOVERE-8372EE0F: F5nzx registered BeginFrame recovered-state scheduler decision

## 概要

F5nzw recovered loop state has no typed scheduler decision boundary, so a recovered state cannot select resume or abort authority safely.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_recovered_state_scheduler_decision.nepl`

## 根拠

- F5nzw `RecoveredState` を一度だけ消費する decision boundary がないため、同じ unsupported support へ無制限に再投入する経路を型で排除できなかった。
- `ResumeSlice` と `Abort` を move-only handoff に分離し、resume helper の呼出権限を `ResumePending` のみに限定した。

## 問題

F5nzw recovered loop state has no typed scheduler decision boundary, so its registered continuation cannot be handed to either resume or abort without an untyped ownership choice.

## 影響

Recovered loop state cannot be resumed or aborted with its registered continuation and diagnostics intact. RetryPending finite retry policy remains a separate F5nzy concern.

## 修正方針

Consume only F5nzw RecoveredState authority and classify a caller-supplied value-only decision into resume-ready or abort-ready move-only handoffs, preserving category and exact diagnostic without executing schedulers or host work.

## 検証

Actual DriverCompletionFailed recovery and resume/abort fixtures, source-policy isolation, normal compile, lower regressions, release/trunk/CLI gates, and subagent reviews.

- release runtime fixture: 2/2、resume evidence 124、abort evidence 60、elapsed 4:38.79、max RSS 375388 KiB
- Web GUI font source-policy、normal-mode test isolation 2件、native/wasm workspace check、release CLI build、trunk build、Playground editor CLI JSON 13/13、issues check、`git diff --check`
- subagent差分review: blocker/majorなし
