---
id: ISS-20260718T111123026Z-F5NZM-REGISTERED-BEGINFRAME-YIELD-SC-A2E1771C
title: "F5nzm registered BeginFrame Yield scheduler resume authority"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-18
updated: 2026-07-18
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_host_action_yield_resume.nepl
---

# ISS-20260718T111123026Z-F5NZM-REGISTERED-BEGINFRAME-YIELD-SC-A2E1771C: F5nzm registered BeginFrame Yield scheduler resume authority

## 概要

F5nzl success retains a typed Yield loop completion beside the registered continuation, but no registered boundary exposes the unresumed Yield authority to the scheduler before resetting the slice.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_host_action_yield_resume.nepl`

## 根拠

- F5nzl actual Offscreen success returns typed F5nc `Yield` beside the registered continuation.
- F5nc `dispatch_loop_state_resume_slice` is the formal helper that delegates slice reset through F5my without exposing F5mw internals.
- The actual registered state has slice counters 1/0 before resume and 0/0 after resume, so scheduler observation must precede reset.

## 問題

F5nzl success retains a typed Yield loop completion beside the registered continuation, but no registered boundary exposes the unresumed Yield authority to the scheduler before resetting the slice.

## 影響

The registered BeginFrame path cannot respect the Yield boundary or resume the next slice without splitting continuation authority or bypassing F5nc.

## 修正方針

Classify all F5nh completion phases into continuation-bearing owners, then allow only the Yield owner to call the formal F5nc resume helper exactly once and return a resumed owner.

## 検証

Actual F5nzl Yield fixture, pre/post resume counters, source policy, normal isolation, regression gates, and subagent reviews.

## 完了

F5nzl successをContinue/Yield/Completedのcontinuation-bearing ownerへ全域分類し、Yield ownerだけをscheduler-visible unresumed authorityとして公開した。Yield-only entryは既存F5nc resume helperを一度呼び、reset済みstateとcontinuationを同居させる。actual fixtureはresume前1/0、resume後0/0、cleanupをevidence 63で確認した。
