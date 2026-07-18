---
id: ISS-20260718T190000000Z-F5NZT-REGISTERED-TERMINAL-COMMAND
title: "F5nzt registered resumed terminal command authority"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-18
updated: 2026-07-18
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_resumed_terminal_command.nepl
---

# F5nzt registered resumed terminal command authority

F5nzs retained the EndFrame continuation beside a Completed schedule authority without advancing the terminal command cursor.

F5nzt gates the schedule phase before consuming the owner, advances the existing command cursor exactly once only for Completed, and preserves updated state beside the opaque terminal step or lower owner-bearing error.
