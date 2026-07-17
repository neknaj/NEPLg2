---
id: ISS-20260717T104946194Z-REGISTERED-BEGINFRAME-STEP-LACKS-FIR-56B88B5F
title: "Registered BeginFrame step lacks first Run command bridge"
area: GUI_FONT
status: open
resolved: false
priority: P1
type: architecture
created: 2026-07-17
updated: 2026-07-17
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_command_cursor_run.nepl
---

# ISS-20260717T104946194Z-REGISTERED-BEGINFRAME-STEP-LACKS-FIR-56B88B5F: Registered BeginFrame step lacks first Run command bridge

## 概要

The registered compositor graph stops at the F5nyy BeginFrame step and cannot advance its RunPending continuation to the first typed Run command.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_command_cursor_run.nepl`

## 根拠

- F5nyyは最初の`BeginFrame`と`RunPending` continuationをstep内に保持するが、registered graphにはそのownerを次のF5mt stepへ渡すadapterがない。
- Run payload constructionとlower F5ms step authorityは既存F5mtが所有するため、registered側はfirst stepからownerをexactly once回収し、次のstep resultを第二十層へ保持するだけにする。
- このissueは最初の`Run`までを対象とし、EndFrame、terminal Completed、F5mu record、host実行を完成条件に含めない。

## 問題

The registered compositor graph stops at the F5nyy BeginFrame step and cannot advance its RunPending continuation to the first typed Run command.

## 影響

Registered rendering cannot preserve the first encoded run through the established F5mt command stream.

## 修正方針

Add F5nyz lossless bridge that consumes the F5nyy step owner exactly once, calls F5mt step exactly once, appends layer20, and stops before EndFrame.

## 検証

Production-derived fixture must prove first Run payload, canonical descriptor, RunPending continuation, cleanup, source policy, normal compile, build, CLI, regression, and reviews.
