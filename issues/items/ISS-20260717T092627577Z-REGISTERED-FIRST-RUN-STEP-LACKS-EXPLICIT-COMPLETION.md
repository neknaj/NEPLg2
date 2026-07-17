---
id: ISS-20260717T092627577Z-REGISTERED-FIRST-RUN-STEP-LACKS-EXPLICIT-COMPLETION
title: "Registered first run step lacks explicit completion"
area: GUI_FONT
status: open
resolved: false
priority: P1
type: architecture
created: 2026-07-17
updated: 2026-07-17
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_completed_step.nepl
---

# ISS-20260717T092627577Z-REGISTERED-FIRST-RUN-STEP-LACKS-EXPLICIT-COMPLETION: Registered first run step lacks explicit completion

## 概要

F5nyu returns the first `RunReady` step, but the registered stroke graph does not retain the explicit terminal `Completed` step.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_completed_step.nepl`

## 根拠

- one-run fixtureのF5nyu successはnext index 1 / total 1のnext ownerを保持するが、terminal `Completed`は次のF5ms callでのみ観測できる。
- first-step ownerからnext ownerを取り出す処理、second step、error recoveryは既存F5ms authorityを使い、registered側で再実装してはならない。

## 問題

The registered stroke compositor chain stops at `RunReady` and cannot prove explicit typed completion.

## 影響

Registered output cannot safely enter later command semantics because terminal run iteration is not yet represented.

## 修正方針

Add F5nyv lossless bridge from F5nyu first-step success through one second F5ms step, preserving explicit `Completed` and all owner-bearing errors.

## 検証

Pending runtime fixture, source-policy, normal isolation, regression, build, CLI and review gates.
