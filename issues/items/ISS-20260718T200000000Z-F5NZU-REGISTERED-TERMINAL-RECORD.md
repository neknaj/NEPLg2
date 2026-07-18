---
id: ISS-20260718T200000000Z-F5NZU-REGISTERED-TERMINAL-RECORD
title: "F5nzu registered resumed terminal F5mu projection"
area: gui-font
status: open
resolved: false
priority: P1
type: architecture
created: 2026-07-18
updated: 2026-07-19
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_resumed_terminal_record.nepl
---

# F5nzu registered resumed terminal F5mu projection

F5nzt retained the updated dispatch-loop state beside an opaque terminal command cursor step without projecting its typed F5mu result.

F5nzu receives the terminal step after the caller consumes the F5nzt owner through its formal parts handoff and borrows that same step into the existing total F5mu projection. The caller retains the step authority through the existing cleanup path; F5nzu adds no second owner.

The production helper and source policy are implemented, but an actual composite F5nzt-to-F5mu runtime fixture causes nonlinear resource analysis and exceeds a 300-second compile timeout at about 1.8 GB RSS. The F5nzt 1023 control fixture must be compiled with CLI `--test-mode`; with that contract restored it compiles in about 339 seconds and its generated Wasm reports evidence 1023 with zero failures. Owner-return summaries take about 173 seconds, dominated by one-pass enumeration of deeply nested Result owner leaves. Integration remains blocked until the control and composite fixtures compile and run within the normal test gate.
