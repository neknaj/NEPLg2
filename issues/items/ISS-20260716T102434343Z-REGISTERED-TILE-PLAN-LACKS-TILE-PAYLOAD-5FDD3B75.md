---
id: ISS-20260716T102434343Z-REGISTERED-TILE-PLAN-LACKS-TILE-PAYLOAD-5FDD3B75
title: "Registered tile plan lacks tile-payload bridge"
area: GUI_FONT
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-07-16
updated: 2026-07-16
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl
---

# ISS-20260716T102434343Z-REGISTERED-TILE-PLAN-LACKS-TILE-PAYLOAD-5FDD3B75: Registered tile plan lacks tile-payload bridge

## 概要

F5nye stops at compositor tile-plan metadata, so the registered stroke path cannot enter the existing checked tile-scoped payload view.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl`

## 根拠

- F5nye successは`GuiRgba8888CompositorTilePlanOwner`で止まり、registered pathから既存F5meへ進むpublic bridgeが無い。
- F5meはlower checked payload prepare、descriptor/plan authority、tile-relative byte read、prepare failure時のplan recoveryを既に所有するためwrapperで再実装すべきでない。
- one-tile planへの`tile_index=1`は正規に到達可能なowner-bearing F5me failureである。

## 問題

F5nye stops at compositor tile-plan metadata, so the registered stroke path cannot enter the existing checked tile-scoped payload view.

## 影響

Registered glyph pixels cannot progress toward the existing RLE pipeline while retaining checked tile payload and recovery authority.

## 修正方針

Add an F5nyf direct public F5nye-to-F5me bridge with staged owner-bearing recovery, success and reachable invalid-tile-index fixtures, source policy, normal compile regression, and consistent docs.

## 検証

Focused success, upstream recovery, and tile-payload prepare recovery fixtures; existing F5nye/F5me regression; source policy; normal compile; trunk/CLI; issues/diff checks; subagent review.

- F5nyf focused doctest 3件、Web source-policy、normal compile isolationを通過した。
- 既存F5nye focused regression、F5me compositor tile-payload regression、trunk build、playground editor JSON 13/13を通過した。
- exact nested prepare/read error、six-way recovery、issues/diff check、subagent implementation/diff/全体整合reviewを通過した。
