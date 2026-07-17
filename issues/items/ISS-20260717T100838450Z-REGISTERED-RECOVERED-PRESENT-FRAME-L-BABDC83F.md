---
id: ISS-20260717T100838450Z-REGISTERED-RECOVERED-PRESENT-FRAME-L-BABDC83F
title: "Registered recovered present frame lacks command-cursor start bridge"
area: GUI_FONT
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-07-17
updated: 2026-07-17
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_command_cursor_start.nepl
---

# ISS-20260717T100838450Z-REGISTERED-RECOVERED-PRESENT-FRAME-L-BABDC83F: Registered recovered present frame lacks command-cursor start bridge

## 概要

The registered compositor graph recovers present-frame authority but cannot enter the existing typed compositor command cursor.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_command_cursor_start.nepl`

## 根拠

- F5nywはcompleted run authorityをmetadata付きpresent-frame ownerへ戻すが、registered graphにはそのownerを既存F5mt startへ渡すadapterがない。
- descriptor-before-start、F5mr run-cursor restart、start-error recoveryは既存F5mtが所有するため、registered側はF5nyw successをexactly once移送し、既存十七層とF5mt startの第十八層を保持する必要がある。
- このissueは`BeginPending` command-cursor authorityまでを対象とし、F5mt step、BeginFrame/Run/EndFrame発行、record/host executionを完成条件に含めない。

## 問題

The registered compositor graph recovers present-frame authority but cannot enter the existing typed compositor command cursor.

## 影響

Registered rendering cannot expose BeginFrame Run EndFrame command semantics through the established F5mt authority.

## 修正方針

Add F5nyx lossless bridge from F5nyw success to exactly one F5mt command-cursor start, preserving layer18 owner-bearing start errors and stopping before stepping.

## 検証

F5nyx production-derived fixtureは`BeginPending`、metadata 263/16/3/1、surface 7、frame 263、expected run count 1、pixel count 16、owner freeをevidence 63でtrunk前後に確認した。upstream F5nyw evidence 31、lower F5mt/F5mr回帰、新規helperのnormal compile隔離、Web source-policy、issues/diff check、trunk build、Playground editor CLI JSON 13/13、subagent差分・runtime・policy/docs reviewを通過した。既存helper全件normal isolationは過去sliceで20分bounded stopとなったため、新規F5nyx helperを同じnormal-mode compiler経路で単独検証した。
