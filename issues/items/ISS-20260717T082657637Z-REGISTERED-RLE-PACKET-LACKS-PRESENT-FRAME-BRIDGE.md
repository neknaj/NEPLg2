---
id: ISS-20260717T082657637Z-REGISTERED-RLE-PACKET-LACKS-PRESENT-FRAME-BRIDGE
title: "Registered RLE packet lacks present-frame bridge"
area: GUI_FONT
status: open
resolved: false
priority: P1
type: architecture
created: 2026-07-17
updated: 2026-07-17
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_present_frame.nepl
---

# ISS-20260717T082657637Z-REGISTERED-RLE-PACKET-LACKS-PRESENT-FRAME-BRIDGE: Registered RLE packet lacks present-frame bridge

## 概要

F5nyr returns a compositor packet owner, but the registered stroke graph cannot enter existing F5mq std present-frame preparation.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_present_frame.nepl`

## 根拠

- F5nyrはmetadata付きpacket ownerを返すが、registered graphにはcaller supplied `SurfaceId`とともにF5mqへ渡すadapterがない。
- frame-id照合、lower present-frame owner construction、packet-owner recoveryは既存F5mqが所有するため、registered側で再実装せずF5nyr successとtyped surface authorityをexactly once移送する必要がある。

## 問題

The registered stroke compositor chain cannot produce the std present-frame owner required by the existing run-cursor path.

## 影響

Registered stroke output remains disconnected from the formal std present continuation despite having a valid packet owner.

## 修正方針

Add F5nys lossless bridge from F5nyr packet success to existing F5mq present-frame preparation, with fixture, policy, docs and gates.

## 検証

Pending runtime fixture, source-policy, normal isolation, regression, build, CLI and review gates.
