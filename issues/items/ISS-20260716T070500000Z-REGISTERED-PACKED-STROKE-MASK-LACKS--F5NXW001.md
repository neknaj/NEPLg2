---
id: ISS-20260716T070500000Z-REGISTERED-PACKED-STROKE-MASK-LACKS--F5NXW001
title: "Registered packed stroke mask lacks a software drain-start owner"
area: GUI_FONT
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-07-16
updated: 2026-07-16
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl
---

# ISS-20260716T070500000Z-REGISTERED-PACKED-STROKE-MASK-LACKS--F5NXW001: Registered packed stroke mask lacks a software drain-start owner

## 概要

F5nxv seals a registered packed-mask resource with its Copy command, but no software drain-start owner can join that complete lifetime authority to a generic RGBA8888 surface without exposing either side.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl`
- `stdlib/alloc/gui/render2d/index.nepl`

## 根拠

- F5nxv deliberately exposes no raw command or split resource accessor.
- A software drain must retain the registered mask resource until every alpha cell has been consumed.
- Surface validation and rect containment must occur before any later pixel write.

## 問題

There is no owner-bearing boundary that consumes the complete F5nxv prepared owner and software surface together and starts a zero-progress drain cursor.

## 影響

Implementing pixel composition directly would either reconstruct registered authority, permit a dangling mask id, or make failure recovery lose the surface or prepared resource.

## 修正方針

Implement F5nxw as software drain-start only. Revalidate prepared metadata, nested packed storage, internal AlphaMaskRect command, software-surface layout, checked rect containment, and exact cell count in that order. On success retain prepared owner, surface, and cell index zero together; on failure retain the original pair.

F5nxw does not read or write pixels and is not transport. F5nxx will perform bounded SourceOver steps, and F5nxy will create completed dirty-region authority.

## 検証

Focused success/rejection/recovery runtime fixtures, source policy, registered module contracts, normal compile, and subagent review pass.
