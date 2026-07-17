---
id: ISS-20260717T094904781Z-REGISTERED-COMPLETED-STEP-LACKS-PRES-56A74D9B
title: "Registered completed step lacks present-frame recovery bridge"
area: GUI_FONT
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-07-17
updated: 2026-07-17
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_present_frame_recovery.nepl
---

# ISS-20260717T094904781Z-REGISTERED-COMPLETED-STEP-LACKS-PRES-56A74D9B: Registered completed step lacks present-frame recovery bridge

## 概要

The registered compositor chain preserves explicit Completed but does not retain the existing typed present-frame recovery result.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_present_frame_recovery.nepl`

## 根拠

- F5nyv successはexplicit `Completed`を保持するが、later command semanticsが必要とするpresent-frame authorityはcompleted step内部のrun-cursor ownerに残っている。
- existing F5ms `gui_rgba8888_compositor_tile_rle_present_run_step_finish_present_frame`だけがcompleted stepを消費し、F5mr/F5mq rewrap resultをlosslessに返す。
- registered側でfinish-owner、F5mr start、F5mq prepareを再実行するとauthorityを分割または重複するため禁止する。

## 問題

The registered compositor chain preserves explicit Completed but does not retain the existing typed present-frame recovery result.

## 影響

The completed registered stream cannot safely enter the later command cursor because present-frame authority and rewrap failure are not represented.

## 修正方針

Add F5nyw lossless bridge from F5nyv completed-step success through exactly one existing F5ms finish-present-frame call, preserving layer17 owner-bearing rewrap errors.

## 検証

F5nyw production-derived fixtureはrecovered present metadata 263/16/3/1、surface 7、frame 263、expected run count 1、pixel count 16、owner freeをevidence 31でtrunk前後に確認した。upstream F5nyv evidence 7、lower F5ms/F5mr回帰、新規helperのnormal compile隔離、Web source-policy、issues/diff check、一時`npm.cmd` shim経由のtrunk build、Playground editor CLI JSON 13/13、subagent差分・runtime・policy/docs reviewを通過した。既存helper全件normal isolationは過去sliceで20分bounded stopとなったため、今回追加したF5nyw helperを同じnormal-mode compiler経路で単独検証した。
