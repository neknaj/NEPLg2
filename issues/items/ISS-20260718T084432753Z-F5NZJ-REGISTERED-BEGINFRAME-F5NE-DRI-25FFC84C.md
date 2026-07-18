---
id: ISS-20260718T084432753Z-F5NZJ-REGISTERED-BEGINFRAME-F5NE-DRI-25FFC84C
title: "F5nzj registered BeginFrame F5ne driver prepare"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-18
updated: 2026-07-18
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_host_execution_driver.nepl
---

# ISS-20260718T084432753Z-F5NZJ-REGISTERED-BEGINFRAME-F5NE-DRI-25FFC84C: F5nzj registered BeginFrame F5ne driver prepare

## 概要

F5nzi owns the registered command continuation and F5nc pending, but the registered path has not entered F5ne driver prepare.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_host_execution_driver.nepl`

## 根拠

- F5nzi は registered command continuation と F5nc pending を単一 owner に保持している。
- 既存 F5ne prepare は pending を消費して driver pending を返すため、continuation authority と同じ transition で移送する必要がある。
- driver pending だけの consuming accessor を公開すると、host action と command continuation が分断される。

## 問題

F5nzi owns the registered command continuation and F5nc pending, but the registered path has not entered F5ne driver prepare.

## 影響

The registered BeginFrame request cannot expose its metadata-preserving host action without splitting continuation authority or bypassing the formal driver.

## 修正方針

Move both F5nzi authorities into one registered F5nzj owner and call existing F5ne prepare exactly once for the pending.

## 検証

Actual runtime fixture, source-policy, dedicated normal isolation, F5ne regression, release build, CLI JSON, and subagent reviews.

## 完了

F5nzi owner の command continuation と pending を同一 transition で各1回取り出し、pending を既存 F5ne prepare へ exactly once 渡す move-only F5nzj owner を追加した。actual fixture は Offscreen BeginFrame、descriptor metadata 263/16/3/1、surface/frame 7/263、run/pixel count 1/16、continuation cleanup を evidence 127 で release trunk 前後に確認した。source-policy、専用 normal isolation、F5ne 回帰、core/std 回帰、release trunk、Playground CLI JSON、subagent review を通過した。
