---
id: ISS-20260717T112109760Z-REGISTERED-ENDFRAME-STEP-LACKS-TERMI-72FA9B6A
title: "Registered EndFrame step lacks terminal Completed bridge"
area: GUI_FONT
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-07-17
updated: 2026-07-17
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_command_cursor_completed.nepl
---

# ISS-20260717T112109760Z-REGISTERED-ENDFRAME-STEP-LACKS-TERMI-72FA9B6A: Registered EndFrame step lacks terminal Completed bridge

## 概要

The registered compositor graph stops at the F5nza EndFrame step and cannot expose the terminal Completed result from its Completed continuation.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_command_cursor_completed.nepl`

## 根拠

- F5nzaは`EndFrame`と`Completed` continuationをstep内に保持するが、registered graphにはterminal resultを得るadapterがない。
- Completed phaseのF5mt stepはlower F5msを再実行せずterminal `Completed`を返すため、registered側はEndFrame stepからownerをexactly once回収し既存F5mtへ渡すだけにする。
- このissueはterminal `Completed`までを対象とし、F5mu record projection、host実行を完成条件に含めない。

## 問題

The registered compositor graph stops at the F5nza EndFrame step and cannot expose the terminal Completed result from its Completed continuation.

## 影響

Registered rendering cannot prove closure of its typed command cursor before record projection.

## 修正方針

Add F5nzb lossless bridge that consumes F5nza step owner exactly once, calls F5mt step exactly once, appends layer22 terminal Completed step, and stops before F5mu.

## 検証

Production-derived fixtureはterminal `Completed`、canonical descriptor metadata 263/16/3/1、surface 7、frame 263、run count 1、pixel count 16、Completed continuation、owner freeをevidence 127でtrunk前後に確認した。upstream F5nza evidence 127、lower F5mt/F5mr回帰、新規helperのnormal compile隔離、Web source-policy、issues/diff check、release trunk build、Playground editor CLI JSON 13/13、subagent差分・runtime・policy/docs reviewを通過した。
