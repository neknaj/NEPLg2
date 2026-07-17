---
id: ISS-20260717T110514879Z-REGISTERED-FIRST-RUN-STEP-LACKS-ENDF-3261BF7C
title: "Registered first Run step lacks EndFrame command bridge"
area: GUI_FONT
status: open
resolved: false
priority: P1
type: architecture
created: 2026-07-17
updated: 2026-07-17
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_command_cursor_end.nepl
---

# ISS-20260717T110514879Z-REGISTERED-FIRST-RUN-STEP-LACKS-ENDF-3261BF7C: Registered first Run step lacks EndFrame command bridge

## 概要

The registered compositor graph stops at the F5nyz first Run step and cannot advance the exhausted RunPending continuation to typed EndFrame.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_command_cursor_end.nepl`

## 根拠

- 未記入

## 問題

The registered compositor graph stops at the F5nyz first Run step and cannot advance the exhausted RunPending continuation to typed EndFrame.

## 影響

Registered rendering cannot close its established command frame stream.

## 修正方針

Add F5nza lossless bridge that consumes F5nyz step owner exactly once, calls F5mt step exactly once, appends layer21 EndFrame step, and stops before terminal Completed.

## 検証

Fixture must prove EndFrame descriptor, Completed continuation phase, cleanup, policies, normal compile, build, CLI, regressions, and reviews.
