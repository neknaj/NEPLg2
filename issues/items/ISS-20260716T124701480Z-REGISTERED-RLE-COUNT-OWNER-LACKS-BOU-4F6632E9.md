---
id: ISS-20260716T124701480Z-REGISTERED-RLE-COUNT-OWNER-LACKS-BOU-4F6632E9
title: "Registered RLE count owner lacks bounded count-step bridge"
area: GUI_FONT
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-07-16
updated: 2026-07-17
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_count_step.nepl
---

# ISS-20260716T124701480Z-REGISTERED-RLE-COUNT-OWNER-LACKS-BOU-4F6632E9: Registered RLE count owner lacks bounded count-step bridge

## 概要

F5nyg stops at the registered compositor RLE count owner, so the registered stroke path cannot advance the existing bounded F5mg count step.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_count_step.nepl`

## 根拠

- public F5nygを1回、既存F5mg bounded stepを1回だけ呼ぶF5nyh bridgeを追加し、outer start errorとinner step error/step authorityをlosslessに保持した。
- production graph外のadapterとproduction-only fixtureでPending→Completed、invalid budget、exact entry recovery、FrameIdInvalid/DirtyOwnerを検証した。

## 問題

F5nyg stops at the registered compositor RLE count owner, so the registered stroke path cannot advance the existing bounded F5mg count step.

## 影響

Registered glyph tile RLE counting cannot make bounded scheduler progress while retaining staged recovery authority.

## 修正方針

Add an F5nyh direct F5nyg-to-F5mg bridge in an isolated extension with staged owner-bearing recovery, honest runtime fixtures, source policy, normal compile regression, and consistent docs.

## 検証

Focused continuation and upstream recovery fixtures; existing F5nyg/F5mg regressions; source policy; normal compile; trunk/CLI; issues/diff checks; subagent review.

Focused F5nyh 3件、F5nyg 3件、F5mg 2件、Web source-policy、normal compile isolation、trunk build、Playground editor JSON 13/13、issues/diff check、subagent差分・全体整合・履歴粒度reviewを通過した。
