---
id: ISS-20260717T042512548Z-REGISTERED-RLE-ENCODE-SEED-LACKS-REA-529ADF36
title: "Registered RLE encode seed lacks ready cursor bridge"
area: GUI_FONT
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-07-17
updated: 2026-07-17
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_encode_cursor.nepl
---

# ISS-20260717T042512548Z-REGISTERED-RLE-ENCODE-SEED-LACKS-REA-529ADF36: Registered RLE encode seed lacks ready cursor bridge

## 概要

F5nyj produces registered encode-seed authority but the registered stroke path cannot enter existing F5mj ready encode cursor.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_encode_cursor.nepl`

## 根拠

- public F5nyjを1回、seed success時だけ既存F5mjを1回呼ぶF5nyk bridgeを追加し、start/step/completed/seed/cursorの五層authorityをlosslessに保持した。
- production graph外のadapterとisolated fixtureでmetadata、run count 3、ready cursor 0/16、payload byte count 64とowner recoveryを検証した。

## 問題

F5nyj produces registered encode-seed authority but the registered stroke path cannot enter existing F5mj ready encode cursor.

## 影響

Registered glyph RLE output cannot advance to the cursor authority required by writer planning.

## 修正方針

Add an F5nyk lossless bridge from public F5nyj success to existing F5mj, preserving all staged errors.

## 検証

Focused reachable cursor success; existing F5nyj staged and F5mj cursor-error regressions; source policy; normal compile; trunk/CLI; issues/diff; subagent review.

Focused F5nyk success、F5nyj回帰、F5mj回帰、Web source-policy、新規helper normal compile isolation、trunk build、Playground editor JSON 13/13、issues/diff check、subagent差分・全体整合reviewを通過した。
