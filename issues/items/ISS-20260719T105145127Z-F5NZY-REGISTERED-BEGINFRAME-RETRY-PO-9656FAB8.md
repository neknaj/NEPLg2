---
id: ISS-20260719T105145127Z-F5NZY-REGISTERED-BEGINFRAME-RETRY-PO-9656FAB8
title: "F5nzy registered BeginFrame retry policy"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-19
updated: 2026-07-19
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_retry_policy.nepl
---

# ISS-20260719T105145127Z-F5NZY-REGISTERED-BEGINFRAME-RETRY-PO-9656FAB8: F5nzy registered BeginFrame retry policy

## 概要

F5nzw RetryPending has no finite retry authority or candidate-support transition boundary.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_retry_policy.nepl`

## 根拠

- F5nzw RetryPendingはregistered continuationとsession pendingを保持するが、再提出許可の有限性とcandidate support検証を保持しなかった。
- actual UnsupportedActionだけを一回許可し、RetryReadyへspent Exhausted budgetを固定して後続へ渡す。その他の診断・budget・decision・candidateは元authorityを保持したtyped abortへ閉じる。

## 問題

F5nzw RetryPending has no finite retry authority or candidate-support transition boundary.

## 影響

A caller could resubmit the same unsupported action indefinitely or lose the registered continuation and exact recovery diagnostic while aborting.

## 修正方針

Consume RetryPending once, allow one candidate-supported resubmit handoff with exhausted carried budget, and otherwise produce a diagnostic-preserving abort handoff without executing completion, scheduler, host, or platform work.

## 検証

Actual unsupported recovery fixture covers supported transition, exhausted budget, unsupported candidate, exact diagnostic preservation, source-policy isolation, normal compile, lower regressions, release/trunk/CLI gates, and subagent reviews.

- release runtime fixture: 5/5、evidence 63/23/71/15/17、elapsed 5:18.19、max RSS 414596 KiB
- actual lower mismatch、rejected OffscreenBegin、inner executor Unsupported category、spent Exhausted budgetを検証
- Web source-policy、normal-mode test isolation、native/wasm check、release CLI build、trunk、Playground editor CLI JSON、issues、diff-check
- subagent再review: blocker/majorなし
