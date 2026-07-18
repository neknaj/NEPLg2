---
id: ISS-20260718T064405912Z-F5NZH-REGISTERED-BEGINFRAME-SCHEDULE-8678241B
title: "F5nzh registered BeginFrame scheduled dispatch adoption bridge"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-18
updated: 2026-07-18
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_dispatch.nepl
---

# ISS-20260718T064405912Z-F5NZH-REGISTERED-BEGINFRAME-SCHEDULE-8678241B: F5nzh registered BeginFrame scheduled dispatch adoption bridge

## 概要

F5nzg retains a successful F5mw BeginFrame schedule step and matching F5mx request, but the registered path is not represented as the existing F5my RequestReady dispatch step.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_dispatch.nepl`

## 根拠

- F5nzg already owns the successful F5mw BeginFrame schedule step and its matching F5mx request.
- Existing F5my dispatch stepping would submit the retained record to F5mw/F5mx again, so it cannot adopt that already-consumed pair without replay.
- The adopter must therefore be a trusted boundary whose callers preserve pair provenance through one move-only owner.

## 問題

F5nzg retains a successful F5mw BeginFrame schedule step and matching F5mx request, but the registered path is not represented as the existing F5my RequestReady dispatch step.

## 影響

The registered glyph compositor cannot enter the formal scheduled dispatch contract without replaying the already-consumed BeginFrame through F5mw/F5mx.

## 修正方針

Add an F5my adopter for an already-successful schedule step and request, then wrap the F5nzg owner with the resulting dispatch step without revalidation, record replay, request reconstruction, or host execution.

## 検証

Focused runtime fixture, source-policy, normal compile isolation, F5my regression, core tests, release trunk build, CLI JSON, and subagent reviews.

## 完了

- Added a trusted F5my adoption boundary for a successful F5mw/F5mx pair without replay or request reconstruction.
- The registered move-only owner preserves the matching F5nzg authority and exposes recovery and cleanup.
- Runtime evidence verifies RequestReady/Yield, Offscreen BeginFrame provenance, recovered InFrame drain state, slice counters 1/0, and cleanup.
- Source-policy, dedicated normal isolation, release build, Playground editor 13/13, and subagent review passed.
