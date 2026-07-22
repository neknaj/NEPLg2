---
id: ISS-20260722T035939321Z-WEB-FONT-BUDGETED-RUN-ACTUAL-POLLING-6CD891C7
title: "Web font budgeted Run actual polling loop transition"
area: stdlib/gui
status: investigating
resolved: false
priority: P1
type: architecture
created: 2026-07-22
updated: 2026-07-22
target: stdlib/platforms/gui/web/font_registered_budgeted_run_polling_loop.nepl
---

# ISS-20260722T035939321Z-WEB-FONT-BUDGETED-RUN-ACTUAL-POLLING-6CD891C7: Web font budgeted Run actual polling loop transition

## 概要

The budgeted Run driver and queue/timer owners are not connected by a production Web polling transition.

## 対象

- `stdlib/platforms/gui/web/font_registered_budgeted_run_polling_loop.nepl`

## 根拠

- production transition、owner-bearing failure、unmatched event返却、zero-delay rejectionは実装済み。
- subagent code review上のBlocker/Majorは解消済み。
- Node 22 WASI harnessがguest終了後にJS assertionへ戻らず、runtime evidenceは未確定。

## 問題

The budgeted Run driver and queue/timer owners are not connected by a production Web polling transition.

## 影響

Ready and Suspended owners can only be manually orchestrated by tests, so immediate reentry and timer-gated resume are not enforced by one production boundary.

## 修正方針

Add a bounded production transition that executes queued Ready owners, immediately requeues Ready outcomes, submits Suspended timers, polls pending timer events, and preserves owner-bearing failures and terminal outcomes.

## 検証

- `ISS-20260722T040952289Z-NODE-22-WASI-RUNTIME-HARNESS-EXITS-B-B4756601` 修正後、actual two-Run runtimeがevidence 12、Begin/Run/End 1/2/0、timer 1、poll 2を返すことを確認する。
- ScheduleFailed evidence 13、PollFailed evidence 14と各cleanupを確認する。
