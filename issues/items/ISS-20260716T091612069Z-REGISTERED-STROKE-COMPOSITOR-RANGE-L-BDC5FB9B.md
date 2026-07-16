---
id: ISS-20260716T091612069Z-REGISTERED-STROKE-COMPOSITOR-RANGE-L-BDC5FB9B
title: "Registered stroke compositor range lacks byte-storage bridge"
area: GUI_FONT
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-07-16
updated: 2026-07-16
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl
---

# ISS-20260716T091612069Z-REGISTERED-STROKE-COMPOSITOR-RANGE-L-BDC5FB9B: Registered stroke compositor range lacks byte-storage bridge

## 概要

F5nyc stops at compositor batch-range metadata, so the registered stroke path cannot reach copied row byte storage.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl`

## 根拠

- F5nyc successは`GuiRgba8888CompositorBatchRangeOwner`で止まり、registered pathから既存F5mc `gui_rgba8888_compositor_byte_storage_prepare`を呼ぶpublic bridgeが無かった。
- F5ma/F5nybはcursorを進めdescriptor authorityを失い得るため、drain terminalからrange/byte pathへ戻す構成は不正である。
- F5mcはmetadata copy、lower row-byte prepare、prepare failure時のrange recoveryを既に所有するため、registered wrapperで再実装する必要はない。

## 問題

F5nyc stops at compositor batch-range metadata, so the registered stroke path cannot reach copied row byte storage.

## 影響

Registered glyph pixels cannot enter the existing tile/RLE/presentation pipeline while preserving owner authority.

## 修正方針

Add F5nyd direct F5nyc-to-F5mc bridge with staged owner-bearing recovery, focused runtime fixtures, source policy, normal compile regression, and consistent docs.

## 検証

Focused success and entry-stage recovery fixtures; existing F5mc contract delegation; source policy; normal compile; issues/diff checks; subagent review.

- F5nyd focused doctest #1/#2、Web GUI source-policy、normal compile isolationを通過した。
- 既存F5mc compositor byte-storage doctest、trunk build、playground editor JSON 13/13を通過した。
- issues index/check、diff check、subagent implementation/diff/全体整合reviewを通過した。
