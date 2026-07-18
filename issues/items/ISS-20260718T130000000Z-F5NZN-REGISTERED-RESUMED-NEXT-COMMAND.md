---
id: ISS-20260718T130000000Z-F5NZN-REGISTERED-RESUMED-NEXT-COMMAND
title: "F5nzn registered resumed next command authority"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-18
updated: 2026-07-18
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_resumed_next_command.nepl
---

# F5nzn registered resumed next command authority

## 概要

F5nzm resumed owner retained the reset F5nc state and registered continuation but lacked a bounded transition to the next command.

## 根拠

- The retained BeginFrame cursor continuation is RunPending after the formal registered wrapper chain is consumed.
- F5mt owns the canonical one-command transition and owner-bearing failure recovery.

## 修正方針

Consume one resumed parts authority, recover the cursor only through formal take APIs, and invoke F5mt exactly once while retaining reset state on both outcomes.

## 完了

F5nzn co-locates reset F5nc state with the next Run step or lower step error and stops before Run recording or further execution.
