---
id: ISS-20260719T093510810Z-F5NZW-REGISTERED-BEGINFRAME-HOST-ACT-89795408
title: "F5nzw registered BeginFrame host action failure recovery"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-19
updated: 2026-07-19
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_host_action_failure_recovery.nepl
---

# ISS-20260719T093510810Z-F5NZW-REGISTERED-BEGINFRAME-HOST-ACT-89795408: F5nzw registered BeginFrame host action failure recovery

## 概要

F5nzl completion errors can only be aborted; registered continuation and lower retry/state recovery authority cannot be resumed together.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_host_action_failure_recovery.nepl`

## 根拠

- F5nzl errorはregistered dispatch owner、category、F5nh lower errorを保持するが、公開操作はabortだけだった。
- F5ng/F5nf/F5neはretry driverまたはrollback stateを保持しており、破棄せず回収できる。

## 問題

F5nzl completion errors can only be aborted; registered continuation and lower retry/state recovery authority cannot be resumed together.

## 影響

Unsupported or failed host action completion either discards recoverable pending/state authority or requires bypassing the typed F5nh/F5ng/F5nf/F5ne recovery graph.

## 修正方針

Consume the F5nzl error once and classify the complete lower graph into move-only retry-pending or recovered-loop-state owners while retaining the registered continuation and diagnostics.

## 検証

Actual unsupported SinkRejected retry recovery, source-policy full lower classification, normal compile isolation, lower regressions, release/trunk/CLI gates, and subagent reviews.

## 完了

- F5nzl errorを一度だけpartsへ分解し、AttemptActionMismatch、SinkRejected、DriverCompletionFailedを詳細lower diagnostic付きRetryPendingまたはRecoveredStateへ全域分類した。
- retry/recovered ownerはregistered continuation、category、diagnostic、session pendingまたはrollback stateをconsuming partsで後続へ渡す。abort freeはpending放棄後にregistered continuationを閉じる。
- actual unsupported fixtureはcategory、SinkRejected/UnsupportedAction、OffscreenBegin retry、cleanupをevidence 60で検証した。runtime 2/2、source-policy、normal compile、native/wasm check、release CLI、trunk build、Playground editor 13/13、subagent差分reviewを通過した。
