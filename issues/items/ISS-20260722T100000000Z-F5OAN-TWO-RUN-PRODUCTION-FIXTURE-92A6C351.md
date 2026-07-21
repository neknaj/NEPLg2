---
id: ISS-20260722T100000000Z-F5OAN-TWO-RUN-PRODUCTION-FIXTURE-92A6C351
title: "F5oan production two-Run fixture"
area: gui-font
status: open
resolved: false
priority: P1
type: architecture
created: 2026-07-22
updated: 2026-07-22
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_two_run_fixture_test.nepl
---

# F5oan production two-Run fixture

registered software-drain completionのsurfaceをconsuming production pixel writerで更新し、既存compositor tile RLE count bridgeが16 pixelからexact 2 runを生成するfixtureを追加する。private completed fields、RLE count、cursor progressは手組みしない。既存Resource診断を越えてruntime assertionが通過するまで未解決とし、actual Web Run phaseへのsupplied-owner接続は後続とする。
