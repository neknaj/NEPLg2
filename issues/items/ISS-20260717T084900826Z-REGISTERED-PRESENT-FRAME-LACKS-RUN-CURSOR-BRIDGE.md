---
id: ISS-20260717T084900826Z-REGISTERED-PRESENT-FRAME-LACKS-RUN-CURSOR-BRIDGE
title: "Registered present frame lacks run-cursor bridge"
area: GUI_FONT
status: open
resolved: false
priority: P1
type: architecture
created: 2026-07-17
updated: 2026-07-17
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_run_cursor.nepl
---

# ISS-20260717T084900826Z-REGISTERED-PRESENT-FRAME-LACKS-RUN-CURSOR-BRIDGE: Registered present frame lacks run-cursor bridge

## 概要

F5nys returns a compositor present-frame owner, but the registered stroke graph cannot enter existing F5mr std run-cursor start.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_run_cursor.nepl`

## 根拠

- F5nysはmetadata付きpresent-frame ownerを返すが、registered graphにはそのauthorityをF5mrへ渡すadapterがない。
- lower run-cursor construction、start failure recovery、metadata preservationは既存F5mrが所有するため、registered側で再実装せずF5nys successをexactly once移送する必要がある。

## 問題

The registered stroke compositor chain cannot produce the std run-cursor owner required by the existing present continuation.

## 影響

Registered stroke output remains disconnected from the formal run iteration path despite having a valid present-frame owner.

## 修正方針

Add F5nyt lossless bridge from F5nys present-frame success to existing F5mr run-cursor start, with fixture, policy, docs and gates.

## 検証

Pending runtime fixture, source-policy, normal isolation, regression, build, CLI and review gates.
