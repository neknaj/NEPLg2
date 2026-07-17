---
id: ISS-20260717T100838450Z-REGISTERED-RECOVERED-PRESENT-FRAME-L-BABDC83F
title: "Registered recovered present frame lacks command-cursor start bridge"
area: GUI_FONT
status: open
resolved: false
priority: P1
type: architecture
created: 2026-07-17
updated: 2026-07-17
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_command_cursor_start.nepl
---

# ISS-20260717T100838450Z-REGISTERED-RECOVERED-PRESENT-FRAME-L-BABDC83F: Registered recovered present frame lacks command-cursor start bridge

## 概要

The registered compositor graph recovers present-frame authority but cannot enter the existing typed compositor command cursor.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_command_cursor_start.nepl`

## 根拠

- 未記入

## 問題

The registered compositor graph recovers present-frame authority but cannot enter the existing typed compositor command cursor.

## 影響

Registered rendering cannot expose BeginFrame Run EndFrame command semantics through the established F5mt authority.

## 修正方針

Add F5nyx lossless bridge from F5nyw success to exactly one F5mt command-cursor start, preserving layer18 owner-bearing start errors and stopping before stepping.

## 検証

Runtime fixture, source-policy, normal isolation, lower regression, trunk build, CLI JSON and subagent review.
