---
id: ISS-20260718T140000000Z-F5NZO-REGISTERED-RESUMED-RUN-RECORD
title: "F5nzo registered resumed Run record authority"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-18
updated: 2026-07-18
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_resumed_run_record.nepl
---

# F5nzo registered resumed Run record authority

## 概要

F5nzn retained the reset loop state and next Run step but did not expose the canonical F5mu record while preserving both authorities.

## 根拠

- F5mu provides the total typed projection from a borrowed F5mt step.
- The next bounded transition must retain reset loop state and the move-only cursor step.

## 完了

F5nzo co-locates reset state, Run step, and Copy RunRecord projection and stops before virtual drain or scheduling.
