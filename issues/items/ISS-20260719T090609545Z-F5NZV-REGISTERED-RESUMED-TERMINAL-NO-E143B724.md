---
id: ISS-20260719T090609545Z-F5NZV-REGISTERED-RESUMED-TERMINAL-NO-E143B724
title: "F5nzv registered resumed terminal no-request completion bridge"
area: gui-font
status: open
resolved: false
priority: P1
type: architecture
created: 2026-07-19
updated: 2026-07-19
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_resumed_terminal_completion.nepl
---

# ISS-20260719T090609545Z-F5NZV-REGISTERED-RESUMED-TERMINAL-NO-E143B724: F5nzv registered resumed terminal no-request completion bridge

## 概要

F5nzu projects the terminal F5mu result but leaves retained dispatch state and terminal step cleanup disconnected from an explicit loop Completed value.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_resumed_terminal_completion.nepl`

## 根拠

- 未記入

## 問題

F5nzu projects the terminal F5mu result but leaves retained dispatch state and terminal step cleanup disconnected from an explicit loop Completed value.

## 影響

The registered resumed terminal path cannot finish without either leaking the terminal step authority or incorrectly routing through request/executor completion.

## 修正方針

Consume F5nzt formal parts once, free the terminal step exactly once through F5mt, return Completed retained state only after cleanup succeeds, and preserve cleanup failure kind with retained state in a typed error.

## 検証

Production-derived 4095 runtime fixture, source-policy, normal compile isolation, lower regression, native/wasm checks, release build, trunk build, CLI JSON, and subagent reviews.
