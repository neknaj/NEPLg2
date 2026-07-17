---
id: ISS-20260717T040516834Z-REGISTERED-COMPLETED-RLE-COUNT-LACKS-39C7A401
title: "Registered completed RLE count lacks encode-seed bridge"
area: GUI_FONT
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-07-17
updated: 2026-07-17
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_encode_seed.nepl
---

# ISS-20260717T040516834Z-REGISTERED-COMPLETED-RLE-COUNT-LACKS-39C7A401: Registered completed RLE count lacks encode-seed bridge

## 概要

F5nyi produces registered completed count evidence but the registered stroke path cannot enter existing F5mi encode seed authority.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_encode_seed.nepl`

## 根拠

- public F5nyiを1回、completed success時だけ既存F5miを1回呼ぶF5nyj bridgeを追加し、start/step/completed/seedの四層authorityをlosslessに保持した。
- production graph外のadapterとisolated fixtureでmetadata、exact run count 3、payload byte count 64とowner recoveryを検証した。

## 問題

F5nyi produces registered completed count evidence but the registered stroke path cannot enter existing F5mi encode seed authority.

## 影響

Registered glyph RLE output cannot advance from exact run-count evidence into encoded payload transport.

## 修正方針

Add an F5nyj lossless bridge from public F5nyi success to existing F5mi, preserving start, step, completed, and seed errors.

## 検証

Focused reachable seed-success fixture; existing F5nyi staged-recovery and F5mi seed-error regressions; source policy; normal compile; trunk/CLI; issues/diff; subagent review.

Focused F5nyj success、F5nyi staged recovery 4経路、F5mi回帰、Web source-policy、新規helper normal compile isolation、trunk build、Playground editor JSON 13/13、issues/diff check、subagent差分・全体整合reviewを通過した。
