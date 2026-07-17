---
id: ISS-20260717T063919545Z-REGISTERED-STROKE-RLE-PATH-LACKS-COM-14D00AE4
title: "Registered stroke RLE path lacks completion probe"
area: GUI_FONT
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-07-17
updated: 2026-07-17
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_write_completed.nepl
---

# ISS-20260717T063919545Z-REGISTERED-STROKE-RLE-PATH-LACKS-COM-14D00AE4: Registered stroke RLE path lacks completion probe

## 概要

F5nyo returns the first WroteRun authority but the registered path cannot prove terminal Completed before F5mo sealing.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_write_completed.nepl`

## 根拠

- F5nyoはproduction-derived 1-run入力のfirst `WroteRun`とnext writer authorityを返すが、registered adapterにはそのauthorityをterminal probeへ渡す境界がない。
- status、progress、owner-bearing error recoveryのauthorityは既存F5mnが所有するため、registered側でwriterを再実装せずF5nyo successから回収したownerをF5mnへexactly once渡す必要がある。

## 問題

F5nyo returns the first WroteRun authority but the registered path cannot prove terminal Completed before F5mo sealing.

## 影響

The registered font-to-GUI owner chain stops before a fully written encoded tile.

## 修正方針

Add F5nyp lossless second-step bridge with runtime fixture, source policy, normal compile, docs, and gates.

## 検証

Focused fixture returns evidence 255 with Completed and all regression gates pass.

確認済み:

- F5nyp focused runtimeはsecond step `Completed`とmetadata/progress evidence 255をtrunk前後に検証した。
- upstream F5nyo evidence 255とlower F5mn success/error recovery回帰を通過した。
- Web source-policy、corrected normal compile isolation、issues/diff check、Trunk build、Playground editor CLI JSON 13/13を通過した。
- subagentのproduction/full diff/compiler/global consistency review指摘を修正した。
