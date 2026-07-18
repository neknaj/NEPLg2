---
id: ISS-20260718T180000000Z-F5NZS-REGISTERED-END-FRAME-SCHEDULE
title: "F5nzs registered resumed EndFrame schedule authority"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-18
updated: 2026-07-18
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_resumed_end_frame_schedule.nepl
---

# F5nzs registered resumed EndFrame schedule authority

F5nzr retained the typed EndFrame record without applying it to the updated InFrame loop authority.

F5nzs validates the EndFrame result, invokes the existing dispatch-loop schedule-only entry exactly once, and retains the cursor beside updated or rollback state without issuing a host request.
