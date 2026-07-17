---
id: ISS-20260717T104946194Z-REGISTERED-BEGINFRAME-STEP-LACKS-FIR-56B88B5F
title: "Registered BeginFrame step lacks first Run command bridge"
area: GUI_FONT
status: verified
resolved: true
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

Production-derived fixtureは最初のRun offset 0/count 16/RGBA 11/22/33/44、canonical descriptor metadata 263/16/3/1、surface 7、frame 263、run count 1、pixel count 16、RunPending continuation、owner freeをevidence 1023でtrunk前後に確認した。upstream F5nyy evidence 127、lower F5mt/F5mr回帰、新規helperのnormal compile隔離、Web source-policy、issues/diff check、release trunk build、Playground editor CLI JSON 13/13、subagent差分・runtime・policy/docs reviewを通過した。
