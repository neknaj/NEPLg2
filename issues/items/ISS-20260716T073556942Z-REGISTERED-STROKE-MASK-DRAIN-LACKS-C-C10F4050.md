---
id: ISS-20260716T073556942Z-REGISTERED-STROKE-MASK-DRAIN-LACKS-C-C10F4050
title: "Registered stroke mask drain lacks checked dirty-region completion authority"
area: GUI_FONT
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-07-16
updated: 2026-07-16
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl
---

# ISS-20260716T073556942Z-REGISTERED-STROKE-MASK-DRAIN-LACKS-C-C10F4050: Registered stroke mask drain lacks checked dirty-region completion authority

## 概要

F5nxx can complete all registered packed-mask pixel writes but its sole drain owner cannot yet become a typed dirty-region completion owner.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl`

## 根拠

- 未記入

## 問題

F5nxx can complete all registered packed-mask pixel writes but its sole drain owner cannot yet become a typed dirty-region completion owner.

## 影響

Without a checked completion boundary, changed pixels cannot enter render2d dirty tracking without splitting prepared resource and surface lifetime or inventing unchecked dirty metadata.

## 修正方針

Implement F5nxy to consume only an F5nxx Completed poll result, rederive and validate the resource rect, construct DirtyRegion with dirty_region_rect_checked, and retain prepared resource, surface, and dirty metadata in one owner-bearing completion result. Reject nonterminal status and dirty failure with recoverable ownership; do not enter transport or platform presentation.

## 検証

Focused completed, nonterminal rejection, checked dirty metadata, recovery fixtures; Web GUI source policy; registered module; normal compile; review.

Focused checked-rect completion, nonterminal recovery, forged cached-count/index mismatch recovery, Web GUI source policy, normal compile, issue validation, and subagent review pass.
