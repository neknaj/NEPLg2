---
id: ISS-20260717T080413117Z-REGISTERED-ENCODED-RLE-OWNER-LACKS-PACKET-BRIDGE
title: "Registered encoded RLE owner lacks packet bridge"
area: GUI_FONT
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-07-17
updated: 2026-07-17
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_packet.nepl
---

# ISS-20260717T080413117Z-REGISTERED-ENCODED-RLE-OWNER-LACKS-PACKET-BRIDGE: Registered encoded RLE owner lacks packet bridge

## 概要

F5nyq returns a sealed encoded owner, but the registered stroke graph cannot enter existing F5mp packet preparation.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_packet.nepl`

## 根拠

- F5nyqはmetadata付きsealed encoded ownerを返すが、registered graphにはそのauthorityをF5mpへ渡すadapterがない。
- packet descriptor validation、packet owner construction、owner-bearing prepare error recoveryは既存F5mpが所有するため、registered側で再実装せずF5nyq successをexactly once移送する必要がある。

## 問題

The registered stroke compositor chain cannot produce the packet owner required by the existing F5mq std present boundary.

## 影響

The registered stroke output remains disconnected from the formal packet-to-present path.

## 修正方針

Add F5nyr lossless bridge from F5nyq encoded success to existing F5mp packet preparation, with fixture, policy, docs and gates.

## 検証

F5nyr production-derived fixtureはmetadataとpacket descriptorのframe/batch/tile、plan/tile range、16x16 surface shape、tiling、pixel count 16、run count 1、encoded bytes 12、owner freeをevidence 255でtrunk前後に確認した。upstream F5nyq evidence 31、lower F5mp回帰、F5nyr test-only helperのnormal compile隔離、Web source-policy、issues/diff check、trunk build、Playground editor CLI JSON 13/13を通過した。既存45 helperの全件normal isolationは直前sliceで20分bounded stopとなったため、今回追加したF5nyr helperを同じnormal-mode compiler経路で単独検証した。
