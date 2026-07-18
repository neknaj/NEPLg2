---
id: ISS-20260718T100152913Z-F5NZL-REGISTERED-BEGINFRAME-F5NH-OUT-5757E3A6
title: "F5nzl registered BeginFrame F5nh outcome completion"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-18
updated: 2026-07-18
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_host_action_completion.nepl
---

# ISS-20260718T100152913Z-F5NZL-REGISTERED-BEGINFRAME-F5NH-OUT-5757E3A6: F5nzl registered BeginFrame F5nh outcome completion

## 概要

F5nzk retains the registered command continuation beside an F5nh session pending, but the registered path cannot complete a caller-supplied executor outcome without splitting those authorities.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_host_action_completion.nepl`

## 根拠

- F5nzk co-locates the registered command continuation with the F5nh session pending.
- F5nh complete is the existing boundary that validates the pending action and routes the caller-supplied outcome through F5ng/F5nf.
- Both accepted completion and owner-bearing rejection must remain associated with the registered continuation before any next-command step.

## 問題

F5nzk retains the registered command continuation beside an F5nh session pending, but the registered path cannot complete a caller-supplied executor outcome without splitting those authorities.

## 影響

The registered BeginFrame dispatch cannot rejoin typed completion or failure with its command continuation before the next command.

## 修正方針

Move the F5nzk continuation and session pending together, call F5nh complete exactly once with caller-supplied support and outcome, and retain the continuation beside both success and error authorities.

## 検証

Actual Yield success and unsupported runtime fixtures with explicit stdout/return-value/exit-code assertions, source-policy, dedicated normal isolation, F5nh regression, release build, CLI JSON, and subagent reviews.

## 完了

F5nzk continuationとF5nh pendingを同時にmoveし、caller-supplied support/outcomeを既存F5nh completeへ一度だけ渡すF5nzl ownerを実装した。successはcontinuationとcompletion、failureはcontinuation、typed category、lower errorを保持する。fixtureはOffscreen successとWindow unsupported typed rejectionの明示abortを検証し、lower recoveryの再開は次sliceへ残す。
