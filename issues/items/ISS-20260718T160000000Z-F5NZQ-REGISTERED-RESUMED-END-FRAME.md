---
id: ISS-20260718T160000000Z-F5NZQ-REGISTERED-RESUMED-END-FRAME
title: "F5nzq registered resumed EndFrame command authority"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-18
updated: 2026-07-18
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_resumed_end_frame.nepl
---

# F5nzq registered resumed EndFrame command authority

## 概要

F5nzp retained the updated InFrame loop state and Run cursor continuation but did not expose the next typed EndFrame command.

## 根拠

- F5mt exclusively owns command-cursor continuation and EndFrame construction.
- Non-Continue schedule phases must retain the cursor without advancing it.

## 完了

F5nzq advances F5mt exactly once only for Continue, retains updated state with typed EndFrame or complete recovery authority, and stops before EndFrame record, host request, or platform execution.
