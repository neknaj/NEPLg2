---
id: ISS-20260718T092619670Z-F5NZK-REGISTERED-BEGINFRAME-F5NH-EXE-1CDE62B9
title: "F5nzk registered BeginFrame F5nh executor session request"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-18
updated: 2026-07-18
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_host_action_executor_session.nepl
---

# ISS-20260718T092619670Z-F5NZK-REGISTERED-BEGINFRAME-F5NH-EXE-1CDE62B9: F5nzk registered BeginFrame F5nh executor session request

## 概要

F5nzj retains the registered command continuation beside F5ne driver pending, but the registered path has not entered the F5nh executor-session request boundary.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_host_action_executor_session.nepl`

## 根拠

- F5nzj already owns the registered command continuation and F5ne driver pending as one move-only authority.
- F5nh start/request is the existing executor-facing boundary that preserves the expected action beside the driver pending.
- Exposing or consuming the session pending without the registered continuation would split completion authority from the command cursor continuation.

## 問題

F5nzj retains the registered command continuation beside F5ne driver pending, but the registered path has not entered the F5nh executor-session request boundary.

## 影響

The registered BeginFrame action cannot be handed to an executor-facing typed pending without splitting continuation authority or bypassing the formal session.

## 修正方針

Move both F5nzj authorities together, call F5nh session start and request exactly once, and retain the command continuation beside the session pending while stopping before outcome completion or platform execution.

## 検証

Actual runtime fixture, source-policy, dedicated normal isolation, F5nh regression, release build, CLI JSON, and subagent reviews.

## 完了

F5nzj ownerのcommand continuationとF5ne driver pendingを同一transitionで各1回moveし、既存F5nh start/requestへexactly onceずつ渡すregistered boundaryを追加した。actual fixtureはF5nh `Action` pending、Offscreen BeginFrame、metadata 263/16/3/1、surface/frame 7/263、run/pixel 1/16、continuation cleanupをevidence 255でrelease trunk前後に確認した。source-policy、専用normal isolation、F5nh回帰、core/std回帰、release trunk、CLI JSON、subagent reviewを通過した。
