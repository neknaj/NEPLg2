---
id: ISS-20260717T090423969Z-REGISTERED-RUN-CURSOR-LACKS-RUN-STEP-BRIDGE
title: "Registered run cursor lacks run-step bridge"
area: GUI_FONT
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-07-17
updated: 2026-07-17
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_run_step.nepl
---

# ISS-20260717T090423969Z-REGISTERED-RUN-CURSOR-LACKS-RUN-STEP-BRIDGE: Registered run cursor lacks run-step bridge

## 概要

F5nyt returns a compositor run-cursor owner, but the registered stroke graph cannot enter existing F5ms std run-step-one.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_run_step.nepl`

## 根拠

- F5nytはmetadata付きrun-cursor ownerを返すが、registered graphにはそのauthorityをF5msへ渡すadapterがない。
- lower step、result/next-owner recovery、metadata preservationは既存F5msが所有するため、registered側で再実装せずF5nyt successをexactly once移送する必要がある。

## 問題

The registered stroke compositor chain cannot advance its formal run cursor through the existing std continuation.

## 影響

Registered stroke output remains disconnected from typed run iteration despite having a valid initial run-cursor owner.

## 修正方針

Add F5nyu lossless bridge from F5nyt run-cursor success to one existing F5ms run step, with fixture, policy, docs and gates.

## 検証

F5nyu production-derived fixtureはfirst `RunReady`、metadata 263/16/3/1、next record index 1、total run count 1、run-step freeをevidence 7でtrunk前後に確認した。upstream F5nyt evidence 7、lower F5ms/F5mr回帰、F5nyu test-only helperのnormal compile隔離、Web source-policy、issues/diff check、一時`npm.cmd` shim経由のtrunk build、Playground editor CLI JSON 13/13、subagent差分・runtime・policy/docs reviewを通過した。既存helper全件normal isolationは過去sliceで20分bounded stopとなったため、今回追加したF5nyu helperを同じnormal-mode compiler経路で単独検証した。
