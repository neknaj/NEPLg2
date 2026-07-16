---
id: ISS-20260716T110516241Z-REGISTERED-TILE-PAYLOAD-LACKS-RLE-COUNT-A3A703A6
title: "Registered tile payload lacks RLE count-start bridge"
area: GUI_FONT
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-07-16
updated: 2026-07-16
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_count_start.nepl
---

# ISS-20260716T110516241Z-REGISTERED-TILE-PAYLOAD-LACKS-RLE-COUNT-A3A703A6: Registered tile payload lacks RLE count-start bridge

## 概要

F5nyf stops at the checked compositor tile payload, so the registered stroke path cannot enter the existing RLE count-start boundary.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_count_start.nepl`
- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl`（test callback seam）

## 根拠

- F5nyf successは`GuiRgba8888CompositorTilePayloadOwner`で止まり、registered pathから既存F5mfへ進むpublic bridgeが無い。
- F5mfはlower cursor/count start、initial progress、metadata preservation、start failure時のpayload recoveryを既に所有するためwrapperで再実装すべきでない。
- 有効なF5nyf payloadからF5mf start failureは自然到達不能なので、failureを偽造せず既存F5mf regressionとsource-policyへ委譲する。

## 問題

F5nyf stops at the checked compositor tile payload, so the registered stroke path cannot enter the existing RLE count-start boundary.

## 影響

Registered glyph tile bytes cannot begin the bounded RLE count pipeline while retaining compositor metadata and recovery authority.

## 修正方針

Add an F5nyg direct public F5nyf-to-F5mf bridge with staged owner-bearing recovery, honest success/upstream-recovery fixtures, source policy, normal compile regression, and consistent docs.

## 検証

Focused success and upstream recovery fixtures; existing F5nyf/F5mf regression; source policy; normal compile; trunk/CLI; issues/diff checks; subagent review.

## 解決

- F5nygを専用extension moduleへ隔離し、public F5nyfと既存F5mfを各1回だけ接続した。
- success、invalid frame、invalid tile indexのruntime fixtureと既存F5nyf/F5mf回帰を通過した。
- source-policy、normal compile、trunk build、playground editor JSON 13/13、subagent reviewを通過した。
