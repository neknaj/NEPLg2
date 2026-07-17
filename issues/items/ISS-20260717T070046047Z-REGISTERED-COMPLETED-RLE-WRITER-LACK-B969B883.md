---
id: ISS-20260717T070046047Z-REGISTERED-COMPLETED-RLE-WRITER-LACK-B969B883
title: "Registered completed RLE writer lacks encoded seal bridge"
area: GUI_FONT
status: open
resolved: false
priority: P1
type: architecture
created: 2026-07-17
updated: 2026-07-17
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_encoded.nepl
---

# ISS-20260717T070046047Z-REGISTERED-COMPLETED-RLE-WRITER-LACK-B969B883: Registered completed RLE writer lacks encoded seal bridge

## 概要

F5nyp proves terminal writer completion but the registered owner graph cannot enter existing F5mo encoded sealing.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_encoded.nepl`

## 根拠

- F5nypはfully-written terminal stepとnext writer authorityを返すが、registered graphにはそのauthorityをF5mo encoded sealへ渡すadapterがない。
- fully-written validation、sealed owner construction、owner-bearing error recoveryは既存F5moが所有するため、registered側でsealを再実装せずF5nyp successから回収したwriterをexactly once移送する必要がある。

## 問題

F5nyp proves terminal writer completion but the registered owner graph cannot enter existing F5mo encoded sealing.

## 影響

The registered stroke compositor chain cannot produce the encoded owner required by F5mp packet preparation.

## 修正方針

Add F5nyq lossless bridge from F5nyp terminal success to existing F5mo seal, with fixture, policy, docs and gates.

## 検証

Production-derived fixture preserves metadata and sealed counts, and all gates pass.
