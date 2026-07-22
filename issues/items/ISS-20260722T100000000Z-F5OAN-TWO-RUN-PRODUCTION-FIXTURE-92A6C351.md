---
id: ISS-20260722T100000000Z-F5OAN-TWO-RUN-PRODUCTION-FIXTURE-92A6C351
title: "F5oan production two-Run fixture"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-22
updated: 2026-07-22
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_two_run_fixture_test.nepl
---

# F5oan production two-Run fixture

registered software-drain completionのsurfaceをconsuming production pixel writerで更新し、既存compositor tile RLE count bridgeが16 pixelからexact 2 runを生成するfixtureを追加した。private completed fields、RLE count、cursor progressは手組みせず、supplied ownerからactual Web Begin / Run1 / Run2へ進む。Run1成功authorityのnext-commandをF5oal genuine Run2 variantへ接続し、production cursor由来のRun2 payloadがpixel offset 15 / count 1であること、actual Web runtimeがBegin 1回 / Run 2回 / End 0回で完走することを固定した。
