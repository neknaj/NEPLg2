---
id: ISS-20260716T071144695Z-REGISTERED-PACKED-STROKE-MASK-LACKS--9CAC050A
title: "Registered packed stroke mask lacks a bounded SourceOver software drain step"
area: GUI_FONT
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-07-16
updated: 2026-07-16
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl
---

# ISS-20260716T071144695Z-REGISTERED-PACKED-STROKE-MASK-LACKS--9CAC050A: Registered packed stroke mask lacks a bounded SourceOver software drain step

## 概要

F5nxw can start a paired drain owner but cannot consume packed alpha cells into the RGBA8888 surface.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl`

## 根拠

- 未記入

## 問題

F5nxw can start a paired drain owner but cannot consume packed alpha cells into the RGBA8888 surface.

## 影響

Without a bounded owner-bearing step, registered glyph stroke pixels cannot reach render2d safely and cursor progress may be detached from write success.

## 修正方針

Implement F5nxx as terminal-first budget 0/1 bounded SourceOver composition with packed alpha read, surface read/write, success-only cursor advance, and owner-bearing recovery. Leave dirty completion to F5nxy.

## 検証

Focused normal, budget, terminal, recovery, pixel composition runtime fixtures; source policy; registered module; normal compile; review.

Focused bounded-step and invalid-budget recovery fixtures, Web GUI source policy, normal compile, issue validation, and subagent review pass.
