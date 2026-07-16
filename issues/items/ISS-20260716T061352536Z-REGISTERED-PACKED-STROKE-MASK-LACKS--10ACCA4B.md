---
id: ISS-20260716T061352536Z-REGISTERED-PACKED-STROKE-MASK-LACKS--10ACCA4B
title: "Registered packed stroke mask lacks a sealed prepared command owner"
area: GUI_FONT
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-07-16
updated: 2026-07-16
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl
---

# ISS-20260716T061352536Z-REGISTERED-PACKED-STROKE-MASK-LACKS--10ACCA4B: Registered packed stroke mask lacks a sealed prepared command owner

## 概要

F5nxu registers packed stroke-mask metadata and preserves its storage owner, but no sealed owner can prepare an alpha-mask render command without allowing the Copy command to outlive its resource.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl`

## 根拠

- F5nxu success pairs a metadata-only table with an owner-bearing registered resource, but it deliberately does not construct a render command.
- A raw `RenderCommand` accessor or arbitrary callback would permit a dangling positive `AlphaMaskId` after the registered packed storage is freed.
- Stored metadata must be checked against the nested reservation before command construction; private-table uniqueness alone does not prove host upload or renderability.

## 問題

No sealed prepared-command lifetime boundary currently keeps the registered packed mask resource and its Copy alpha-mask command inseparable.

## 影響

The later drain/transport phase cannot consume a validated registered-mask command without either reconstructing authority or risking a dangling command id.

## 修正方針

Implement F5nxv as a sealed non-Clone/non-Copy owner that revalidates the F5nxu stored record against the nested reservation, constructs `RenderCommand::AlphaMaskRect` only after equality, and exposes no raw command escape.

## 検証

Focused success/mismatch/recovery runtime fixtures, no-command-escape source policy, registered-module 50/50 regression, normal compile, and subagent review pass.
