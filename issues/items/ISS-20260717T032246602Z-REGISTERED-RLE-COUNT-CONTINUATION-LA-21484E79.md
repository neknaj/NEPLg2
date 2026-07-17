---
id: ISS-20260717T032246602Z-REGISTERED-RLE-COUNT-CONTINUATION-LA-21484E79
title: "Registered RLE count continuation lacks completed evidence bridge"
area: GUI_FONT
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-07-17
updated: 2026-07-17
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_count_completed.nepl
---

# ISS-20260717T032246602Z-REGISTERED-RLE-COUNT-CONTINUATION-LA-21484E79: Registered RLE count continuation lacks completed evidence bridge

## 概要

F5nyh returns a bounded count step but the registered stroke path cannot promote its recovered count owner through existing F5mh completed evidence.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_count_completed.nepl`

## 根拠

- public F5nyhを1回、step success時だけ既存F5mhを1回呼ぶF5nyi bridgeを追加し、start/step/completedの三層authorityをlosslessに保持した。
- production graph外のadapterとisolated fixtureでcompleted run count 3/cursor 16、Pending count owner recovery、invalid budget、FrameIdInvalid/DirtyOwnerを検証した。

## 問題

F5nyh returns a bounded count step but the registered stroke path cannot promote its recovered count owner through existing F5mh completed evidence.

## 影響

Registered glyph RLE counting cannot produce exact completed run-count authority for the later encode seed boundary.

## 修正方針

Add an F5nyi triple-nested lossless bridge that calls public F5nyh once and existing F5mh once only on step success, preserving start, step, and completed errors.

## 検証

Focused completed, pending, invalid-budget, and invalid-frame authority fixtures; F5nyh/F5mh regressions; source policy; normal compile; trunk/CLI; issues/diff; subagent review.

Focused F5nyi 4経路、F5nyh/F5mh回帰、Web source-policy、normal compile isolation、trunk build、Playground editor JSON 13/13、issues/diff check、subagent差分・全体整合reviewを通過した。
