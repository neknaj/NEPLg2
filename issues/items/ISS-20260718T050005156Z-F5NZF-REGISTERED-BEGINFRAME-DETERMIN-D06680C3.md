---
id: ISS-20260718T050005156Z-F5NZF-REGISTERED-BEGINFRAME-DETERMIN-D06680C3
title: "F5nzf registered BeginFrame deterministic schedule bridge"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-18
updated: 2026-07-18
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_schedule.nepl
---

# ISS-20260718T050005156Z-F5NZF-REGISTERED-BEGINFRAME-DETERMIN-D06680C3: F5nzf registered BeginFrame deterministic schedule bridge

## 概要

F5nze stops at an already-consumed F5mv BeginFrame virtual-drain owner; the registered stroke path is not yet represented as an F5mw deterministic schedule state.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_schedule.nepl`

## 根拠

- F5nzeはBeginFrame recordを既にF5mvへexactly once消費して`InFrame` drainを所有する。
- F5mwの既存public APIはempty initial stateとrecord stepだけで、validated drainをcounter authority付きstateへ引き継ぐ境界がない。
- 同じBeginFrame recordの再投入はF5mv `BeginAfterBegin`となり、validation authorityも二重化する。

## 問題

F5nze stops at an already-consumed F5mv BeginFrame virtual-drain owner; the registered stroke path is not yet represented as an F5mw deterministic schedule state.

## 影響

The registered glyph compositor cannot advance toward scheduled host continuation while preserving the existing F5mv validation authority and exact slice counters.

## 修正方針

Add a non-bypassing owner bridge from the F5nze authority into the existing F5mw deterministic slice schedule contract without replaying the BeginFrame record.

## 検証

Focused runtime fixture, source-policy, normal compile isolation, lower F5mv/F5mw regression, release trunk build, CLI JSON, subagent reviews.

## 完了

- F5mwへvalidated BeginFrame drain専用adoption APIを追加し、`InFrame`、seen counters 0/0、policyを検査してslice counters 1/0を確立した。
- F5nzf move-only owner/errorがF5nze authority全体とF5mw step/errorを保持し、record replayやF5mv二重stepなしでscheduleへ接続する。
- actual registered pipeline fixtureでbudget 2の`Continue`、budget 1の`Yield`、counter 1/0、InFrame、BeginFrame、RunPending、cleanupをevidence 255として検証した。
- runtime 22/22をrelease trunk前後、Web source-policy、normal isolation、core 887/887、release trunk、Playground editor CLI 13/13、subagent最終reviewを通過した。
