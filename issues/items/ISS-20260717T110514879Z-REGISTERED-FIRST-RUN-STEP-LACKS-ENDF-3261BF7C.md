---
id: ISS-20260717T110514879Z-REGISTERED-FIRST-RUN-STEP-LACKS-ENDF-3261BF7C
title: "Registered first Run step lacks EndFrame command bridge"
area: GUI_FONT
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-07-17
updated: 2026-07-17
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_command_cursor_end_frame.nepl
---

# ISS-20260717T110514879Z-REGISTERED-FIRST-RUN-STEP-LACKS-ENDF-3261BF7C: Registered first Run step lacks EndFrame command bridge

## 概要

The registered compositor graph stops at the F5nyz first Run step and cannot advance the exhausted RunPending continuation to typed EndFrame.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_command_cursor_end_frame.nepl`

## 根拠

- F5nyzは最初の`Run`と`RunPending` continuationをstep内に保持するが、registered graphにはそのownerを次のF5mt stepへ渡すadapterがない。
- lower run cursorは既にexhaustedしているため、既存F5mtだけが同じpublic stepで`EndFrame`を構築し、next ownerを`Completed`へ遷移させるauthorityを持つ。
- このissueは`EndFrame`までを対象とし、次のterminal `Completed`、F5mu record、host実行を完成条件に含めない。

## 問題

The registered compositor graph stops at the F5nyz first Run step and cannot advance the exhausted RunPending continuation to typed EndFrame.

## 影響

Registered rendering cannot close its established command frame stream.

## 修正方針

Add F5nza lossless bridge that consumes F5nyz step owner exactly once, calls F5mt step exactly once, appends layer21 EndFrame step, and stops before terminal Completed.

## 検証

Production-derived fixtureはEndFrame payload/canonical descriptorのmetadata 263/16/3/1、surface 7、frame 263、run count 1、pixel count 16、Completed continuation、owner freeをevidence 127でtrunk前後に確認した。upstream F5nyz evidence 1023、lower F5mt/F5mr回帰、新規helperのnormal compile隔離、Web source-policy、issues/diff check、release trunk build、Playground editor CLI JSON 13/13、subagent差分・runtime・policy/docs reviewを通過した。
