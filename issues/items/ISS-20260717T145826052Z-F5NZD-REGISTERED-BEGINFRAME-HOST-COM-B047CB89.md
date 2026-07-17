---
id: ISS-20260717T145826052Z-F5NZD-REGISTERED-BEGINFRAME-HOST-COM-B047CB89
title: "F5nzd registered BeginFrame host-command record projection"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-17
updated: 2026-07-17
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_record.nepl
---

# ISS-20260717T145826052Z-F5NZD-REGISTERED-BEGINFRAME-HOST-COM-B047CB89: F5nzd registered BeginFrame host-command record projection

## 概要

The registered BeginFrame command step is not retained with its existing F5mu typed record projection.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_record.nepl`

## 根拠

- F5nyyはactual registered pipelineから十九層Result内のBeginFrame command step authorityを返す。
- F5muはborrowed command stepをtyped host-command resultへprojectionする既存境界である。

## 問題

The registered BeginFrame command step is not retained with its existing F5mu typed record projection.

## 影響

The registered stroke compositor path cannot expose an actual typed BeginFrame record authority before virtual drain.

## 修正方針

Borrow the opaque F5nyy BeginFrame step into F5mu exactly once and retain the move-only step plus Copy BeginFrame record result without entering F5mv or host execution.

## 検証

Focused runtime fixture, source policy, normal compile isolation, lower regressions, trunk build, CLI JSON, and subagent reviews.

## 完了

- 十九層Resultをlosslessに保ち、success stepだけをF5muへexactly once borrowするproduction bridgeを追加した。
- move-only stepとCopy `Record(BeginFrame)` resultを第二十authority ownerへ保持し、F5mv、host execution、schedulerより前で停止した。
- fixtureはactual `Record(BeginFrame)`、canonical metadata、surface 7、frame 263、run/pixel count 1/16、元stepのBeginFrame、continuation `RunPending`、cleanupをevidence 127で検証した。
- source-policy、専用module normal isolation、887 core tests、release trunk、release後runtime、CLI JSON 13/13、diff/全体整合reviewを通過した。
