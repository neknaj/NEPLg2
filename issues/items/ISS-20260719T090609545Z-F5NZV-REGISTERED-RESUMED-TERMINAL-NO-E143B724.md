---
id: ISS-20260719T090609545Z-F5NZV-REGISTERED-RESUMED-TERMINAL-NO-E143B724
title: "F5nzv registered resumed terminal no-request completion bridge"
area: gui-font
status: fixed
resolved: true
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

- F5nzt formal partsはretained loop stateとterminal stepを保持し、F5nzuはstepをborrow projectionするだけなので、completion boundaryが同じpartsを一度だけ消費してcleanup authorityを閉じる必要がある。
- terminal `Completed`はhost requestの成功ではないため、F5nc `complete_request`やexecutor outcomeを経由すると存在しないrequest/outcome authorityを捏造する。

## 問題

F5nzu projects the terminal F5mu result but leaves retained dispatch state and terminal step cleanup disconnected from an explicit loop Completed value.

## 影響

The registered resumed terminal path cannot finish without either leaking the terminal step authority or incorrectly routing through request/executor completion.

## 修正方針

Consume F5nzt formal parts once, free the terminal step exactly once through F5mt, return Completed retained state only after cleanup succeeds, and preserve cleanup failure kind with retained state in a typed error.

## 検証

Production-derived 4095 runtime fixture, source-policy, normal compile isolation, lower regression, native/wasm checks, release build, trunk build, CLI JSON, and subagent reviews.

## 2026-07-19 対応結果

F5nzt formal partsからstateを保持し、terminal stepを既存F5mt cleanupへexactly once渡した。cleanup成功後だけ`DispatchLoopCompletion::Completed state`を返し、失敗はretained stateとlower encoded-finish kindをtyped errorへ保持する。normal `complete_request`、request/outcome、F5mv/F5mx、executor、host/platform executionは追加していない。

production-derived fixtureはF5nzu borrowed projection後に同じpartsをF5nzvへ渡し、state 2/16、drain Ended、F5mu/loop Completed、cleanupをevidence 4095 / failed 0で確認した。release CLI WASI gateはelapsed 4:41.91、最大RSS 403688 KiBだった。source-policy、normal compile isolation、native/wasm check、release CLI build、release trunk、Playground editor 13/13、issues/diff check、subagent reviewも通過した。
