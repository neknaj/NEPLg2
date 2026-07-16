---
id: ISS-20260716T094750836Z-REGISTERED-STROKE-BYTE-STORAGE-LACKS-F4B3D0E1
title: "Registered stroke byte storage lacks tile-plan bridge"
area: GUI_FONT
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-07-16
updated: 2026-07-16
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl
---

# ISS-20260716T094750836Z-REGISTERED-STROKE-BYTE-STORAGE-LACKS-F4B3D0E1: Registered stroke byte storage lacks tile-plan bridge

## 概要

F5nyd stops at copied compositor row bytes, so the registered stroke path cannot enter the existing checked tile-plan metadata boundary.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl`

## 根拠

- F5nyd successは`GuiRgba8888CompositorByteStorageOwner`で止まり、registered pathから既存F5mdへ進むpublic bridgeが無かった。
- F5mdはmetadata copy、lower checked tile-plan prepare、descriptor semantics、prepare failure時のstorage recoveryを既に所有するためwrapperで再実装すべきでない。
- `tile_rows=0`は正規のF5nyd storageから到達可能なowner-bearing F5md failureであり、registered recovery契約をruntimeで検証できる。

## 問題

F5nyd stops at copied compositor row bytes, so the registered stroke path cannot enter the existing checked tile-plan metadata boundary.

## 影響

Registered glyph pixels cannot progress toward tile payload/RLE/presentation while retaining compositor metadata and recovery authority.

## 修正方針

Add F5nye direct public F5nyd-to-F5md bridge with staged owner-bearing recovery, success and reachable invalid-tile-config fixtures, source policy, normal compile regression, and consistent docs.

## 検証

Focused success and tile-plan recovery fixtures; existing F5md regression; source policy; normal compile; trunk/CLI; issues/diff checks; subagent review.

- F5nye focused doctest 3件、Web source-policy、normal compile isolationを通過した。
- 既存F5md compositor tile-plan doctest、trunk build、playground editor JSON 13/13を通過した。
- issues index/check、diff check、subagent implementation/diff/全体整合・履歴粒度reviewを通過した。
