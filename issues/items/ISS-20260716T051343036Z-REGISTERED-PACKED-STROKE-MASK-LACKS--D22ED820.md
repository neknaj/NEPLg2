---
id: ISS-20260716T051343036Z-REGISTERED-PACKED-STROKE-MASK-LACKS--D22ED820
title: "Registered packed stroke mask lacks resource reservation"
area: GUI_FONT
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-07-16
updated: 2026-07-16
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl
---

# ISS-20260716T051343036Z-REGISTERED-PACKED-STROKE-MASK-LACKS--D22ED820: Registered packed stroke mask lacks resource reservation

## 概要

F5nxs owns normalized registered stroke alpha cells but no owner-bearing AlphaMaskId reservation binds that storage to placement and paint metadata.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl`

## 根拠

- F5nxs completed packed owner全体を唯一のauthorityとして保持するowner-bearing reservationを実装した。
- positive `AlphaMaskId`、shape全不変条件、alpha storage、SourceOverを順に検証し、失敗時はownerとconfigを回収できる。
- rectはcaller originとowner shapeからのみ導出し、legacy reservation、resource table、command、platform境界を再構築しない。
- focused normal/recovery runtime、normal compile、source-policy、module regression、review gateで検証した。

## 問題

F5nxs owns normalized registered stroke alpha cells but no owner-bearing AlphaMaskId reservation binds that storage to placement and paint metadata.

## 影響

The registered stroke path cannot enter resource registration or prepared render commands without dangling mask identifiers or legacy authority reconstruction.

## 修正方針

Add F5nxt reservation with the whole F5nxs owner as sole authority, positive AlphaMaskId, rect derived from caller origin plus owner shape, caller paint, SourceOver-only validation, owner-bearing recovery, runtime fixture, source policy, normal compile regression, and docs.

## 検証

Focused runtime covers valid reservation and recoverable invalid id/blend/storage invariants; source-policy, normal compile, module regressions, reviews, and integration gates pass.
