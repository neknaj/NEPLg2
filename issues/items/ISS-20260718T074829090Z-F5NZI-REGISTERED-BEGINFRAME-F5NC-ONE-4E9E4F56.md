---
id: ISS-20260718T074829090Z-F5NZI-REGISTERED-BEGINFRAME-F5NC-ONE-4E9E4F56
title: "F5nzi registered BeginFrame F5nc one-shot loop adoption"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-18
updated: 2026-07-18
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_dispatch_loop.nepl
---

# ISS-20260718T074829090Z-F5NZI-REGISTERED-BEGINFRAME-F5NC-ONE-4E9E4F56: F5nzi registered BeginFrame F5nc one-shot loop adoption

## 概要

F5nzh retains an already-successful F5my RequestReady step, but the registered path is not represented as an F5nc one-shot pending loop step.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_dispatch_loop.nepl`

## 根拠

- F5nzh already owns one successful F5my RequestReady step derived from the registered BeginFrame authority.
- Existing F5nc `step_record` would replay F5my/F5mw/F5mx, so it cannot consume that already-successful step.
- Host failure rollback must use the pre-BeginFrame WaitingBegin initial state rather than reverse-engineering the InFrame post-state.

## 問題

F5nzh retains an already-successful F5my RequestReady step, but the registered path is not represented as an F5nc one-shot pending loop step.

## 影響

The registered BeginFrame path cannot enter the formal loop outcome boundary without replaying F5my/F5mw/F5mx or losing rollback provenance.

## 修正方針

Add an initial-BeginFrame trusted F5nc adopter and a move-only registered owner that retains the F5nzh command continuation authority with the non-Copy pending. Expose borrowed pending observation and abort recovery only; the next F5ne bridge must move both authorities together.

## 検証

Focused runtime fixture, source-policy, dedicated normal isolation, F5nc regression, release build, CLI JSON, and subagent reviews.

## 完了

F5nzh command continuation authorityとF5nc pendingを分離せず保持するregistered ownerを追加した。actual fixtureはWaitingBegin 0/0、InFrame 1/0、Offscreen BeginFrame、Yield、continuation recovery/cleanupを確認し、source-policy、normal isolation、F5nc/core/std回帰、release build、CLI JSON、subagent reviewを通過した。
