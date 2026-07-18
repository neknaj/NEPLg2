---
id: ISS-20260718T061210672Z-F5NZG-REGISTERED-BEGINFRAME-HOST-CON-F2E787C0
title: "F5nzg registered BeginFrame host continuation request bridge"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-18
updated: 2026-07-18
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_host_request.nepl
---

# ISS-20260718T061210672Z-F5NZG-REGISTERED-BEGINFRAME-HOST-CON-F2E787C0: F5nzg registered BeginFrame host continuation request bridge

## 概要

F5nzf stops at a registered BeginFrame deterministic schedule owner and does not yet expose the actual F5mu BeginFrame record as an F5mx host continuation request.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_host_request.nepl`

## 根拠

- 未記入

## 問題

F5nzf stops at a registered BeginFrame deterministic schedule owner and does not yet expose the actual F5mu BeginFrame record as an F5mx host continuation request.

## 影響

The registered glyph compositor cannot cross the validated schedule boundary toward a formal host target request while retaining the complete move-only pipeline authority.

## 修正方針

Consume the F5nzf owner, borrow its retained actual F5mu record without reconstruction, call the existing F5mx request constructor exactly once, and retain the whole owner on success and failure while stopping before host execution.

## 検証

Focused runtime fixture, source-policy, normal compile isolation, lower F5mx regression, release trunk build, CLI JSON, and subagent reviews.

## 完了

- F5nze/F5nzf delegate accessorでretained actual F5mu projectionをborrow-copyし、recordを再構築しないF5nzg owner boundaryを追加した。
- supported Offscreen requestとunsupported host errorの双方でF5nzf authority全体、BeginFrame record、schedule Yield、cleanupを検証した。
- focused runtime、Web source-policy、専用normal isolation、core 887/887、release trunk build、Playground editor CLI 13/13、subagent reviewを通過した。
