---
id: ISS-20260716T054858471Z-REGISTERED-PACKED-STROKE-MASK-LACKS--F0A84E95
title: "Registered packed stroke mask lacks metadata-only resource table registration"
area: GUI_FONT
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-07-16
updated: 2026-07-16
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl
---

# ISS-20260716T054858471Z-REGISTERED-PACKED-STROKE-MASK-LACKS--F0A84E95: Registered packed stroke mask lacks metadata-only resource table registration

## 概要

F5nxt reservation binds a registered packed stroke mask to a positive AlphaMaskId, but no metadata-only resource table records the id while preserving storage ownership.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl`

## 根拠

- F5nxt reservation owner は positive mask id と completed F5nxs packed owner を結び付けるが、private table 内での id 重複を拒否する登録境界を持たない。
- metadata record と packed storage authority を同じ lifetime boundary で返す型がなく、後続 prepared command が dangling id を避けるための registered resource owner を取得できない。
- F5nxuでCopy metadata table、registered resource、paired success/error recoveryを実装し、duplicateとactual capacity-zero push failureをruntimeで検証した。

## 問題

F5nxt reservation binds a registered packed stroke mask to a positive AlphaMaskId, but no metadata-only resource table records the id while preserving storage ownership.

## 影響

Prepared commands cannot safely establish registered-mask lifetime or reject duplicate ids without risking dangling AlphaMaskId metadata.

## 修正方針

Implement F5nxu as a metadata-only Copy record table paired with an owner-bearing registered resource, with duplicate-before-push checks and pair-shaped success/error recovery.

## 検証

Focused runtime fixtures cover success, duplicate rejection, push failure recovery, and pair continuation; source policy, module regression, normal compile, and review pass.
