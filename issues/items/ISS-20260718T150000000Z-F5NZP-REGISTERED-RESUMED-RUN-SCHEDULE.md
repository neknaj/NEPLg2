---
id: ISS-20260718T150000000Z-F5NZP-REGISTERED-RESUMED-RUN-SCHEDULE
title: "F5nzp registered resumed Run schedule authority"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-18
updated: 2026-07-18
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_resumed_run_schedule.nepl
---

# F5nzp registered resumed Run schedule authority

## 概要

F5nzo retained a RunRecord and reset loop state but lacked a schedule-only transition into the existing InFrame drain authority.

## 根拠

- F5mw owns validation, F5mv drain stepping, counters, and phase selection.
- Host request construction must remain a separate later boundary.

## 完了

F5nzp invokes F5mw exactly once through F5nc, retains cursor and updated/rollback loop authority, and stops before host request or next command.
