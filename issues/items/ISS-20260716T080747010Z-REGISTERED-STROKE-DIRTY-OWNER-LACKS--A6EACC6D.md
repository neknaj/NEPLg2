---
id: ISS-20260716T080747010Z-REGISTERED-STROKE-DIRTY-OWNER-LACKS--A6EACC6D
title: "Registered stroke dirty owner lacks compositor frame entry bridge"
area: GUI_FONT
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-07-16
updated: 2026-07-16
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl
---

# ISS-20260716T080747010Z-REGISTERED-STROKE-DIRTY-OWNER-LACKS--A6EACC6D: Registered stroke dirty owner lacks compositor frame entry bridge

## 概要

F5nxz returns a generic render2d dirty owner but the registered stroke path cannot enter the existing compositor frame-entry owner without caller-side lifetime splitting.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl`

## 根拠

- F5nxzのgeneric dirty ownerと既存F5lz compositor frame entryは個別にowner-bearingであるが、registered completed ownerから両境界を固定順で通すAPIがなく、callerがlifetime段階を手動接続する必要があった。

## 問題

F5nxz returns a generic render2d dirty owner but the registered stroke path cannot enter the existing compositor frame-entry owner without caller-side lifetime splitting.

## 影響

Registered stroke output remains disconnected from the established bitmap/row compositor pipeline despite preserving dirty metadata.

## 修正方針

Add an owner-bearing registered bridge that converts completed owner through F5nxz and existing compositor-frame-entry prepare, preserving the original completed owner on dirty aggregation failure and the generic dirty owner on lower entry failure; do not enter batch drain or transport.

## 検証

Focused success, dirty aggregation recovery, and compositor prepare recovery fixtures; Web GUI source policy; normal compile; issue/diff checks; subagent review.

Focused successとaggregation/frame-prepareの両recovery doctest、Web GUI source-policy、normal compile isolation、issues/diff check、subagent差分/全体整合reviewを通過した。
