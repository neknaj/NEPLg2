---
id: ISS-20260611T164150959Z-CANVAS-EDITOR-RENDER-AND-MOVEMENT-PA-EB04351C
title: "Canvas editor render and movement paths still scale with whole document on cursor-only work"
area: tools
status: open
resolved: false
priority: P1
type: performance
created: 2026-06-11
updated: 2026-06-11
target: "web/src/editor/editor-renderer.ts; web/src/editor-core/reducer.ts; web/src/editor/editor.ts"
---

# ISS-20260611T164150959Z-CANVAS-EDITOR-RENDER-AND-MOVEMENT-PA-EB04351C: Canvas editor render and movement paths still scale with whole document on cursor-only work

## 概要

Rendering recalculates global line layout and scans occurrence/bracket arrays per visible character. Core vertical/page cursor movement rebuilds line arrays from text on every keypress, bypassing editor line caches.

## 対象

- `web/src/editor/editor-renderer.ts; web/src/editor-core/reducer.ts; web/src/editor/editor.ts`

## 根拠

- 未記入

## 問題

Rendering recalculates global line layout and scans occurrence/bracket arrays per visible character. Core vertical/page cursor movement rebuilds line arrays from text on every keypress, bypassing editor line caches.

## 影響

Cursor blink, ArrowUp/Down, PageUp/Down, and high-occurrence highlighting remain slow on large files even after semantic analysis debounce.

## 修正方針

Make line layout and line starts dirty-versioned caches, convert occurrence/bracket highlights into line segment caches, and move vertical cursor operations to cached line index data.

## 検証

Add synthetic large-file tests for cursor blink/render and vertical movement proving no full text split or per-character occurrence scan happens on cursor-only updates.
