---
id: ISS-20260716T075144185Z-REGISTERED-STROKE-DIRTY-COMPLETION-L-27E1B838
title: "Registered stroke dirty completion lacks generic render2d dirty-owner bridge"
area: GUI_FONT
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-07-16
updated: 2026-07-16
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl
---

# ISS-20260716T075144185Z-REGISTERED-STROKE-DIRTY-COMPLETION-L-27E1B838: Registered stroke dirty completion lacks generic render2d dirty-owner bridge

## 概要

F5nxy seals prepared resource, software surface, and checked DirtyRegion but cannot transfer the surface and dirty metadata into the generic render2d dirty owner.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl`

## 根拠

- 未記入

## 問題

F5nxy seals prepared resource, software surface, and checked DirtyRegion but cannot transfer the surface and dirty metadata into the generic render2d dirty owner.

## 影響

Without an owner-bearing bridge, registered stroke output cannot enter the existing compositor frame pipeline without splitting lifetime authority or dropping dirty metadata.

## 修正方針

Implement F5nxz to aggregate the F5nxy checked DirtyRegion before consuming prepared lifetime, then finish the surface into GuiRgba8888SoftwareSurfaceDirtyOwner. Preserve the F5nxy completed owner on aggregation failure; do not enter compositor frame preparation or transport.

## 検証

Focused success and aggregation recovery fixtures; Web GUI source policy; registered module; render2d regression; normal compile; review.

Focused exact dirty-set success and forced aggregation recovery, Web GUI source policy, normal compile, issue validation, and subagent review pass.
