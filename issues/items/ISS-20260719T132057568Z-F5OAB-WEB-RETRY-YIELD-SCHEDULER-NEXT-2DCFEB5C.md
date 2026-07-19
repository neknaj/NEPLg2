---
id: ISS-20260719T132057568Z-F5OAB-WEB-RETRY-YIELD-SCHEDULER-NEXT-2DCFEB5C
title: "F5oab Web retry Yield scheduler next-command handoff"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-19
updated: 2026-07-19
target: stdlib/platforms/gui/web/font_registered_begin_frame_retry_yield_scheduler.nepl
---

# ISS-20260719T132057568Z-F5OAB-WEB-RETRY-YIELD-SCHEDULER-NEXT-2DCFEB5C: F5oab Web retry Yield scheduler next-command handoff

## 概要

F5oaa retains an unresumed successful Web retry Yield but has no typed scheduler decision or path into the existing F5nzm resume and F5nzn next-command continuation.

## 対象

- `stdlib/platforms/gui/web/font_registered_begin_frame_retry_yield_scheduler.nepl`

## 根拠

- F5oaa successful retry Yield owner is the single authority for the unresumed dispatch slice and its retained retry provenance.
- F5nzm already owns the one-slice resume contract, and F5nzn already owns the following one-command transition; this slice composes those contracts without rebuilding either owner.

## 問題

F5oaa retains an unresumed successful Web retry Yield but has no typed scheduler decision or path into the existing F5nzm resume and F5nzn next-command continuation.

## 影響

The actual Web success path stops after phase classification and cannot preserve retry provenance while resuming one slice or aborting.

## 修正方針

Consume the concrete F5oaa Yield owner into ResumePending or AbortReady, and let ResumePending alone call the existing F5nzm resume and F5nzn next-command helpers exactly once while retaining prior diagnostic and spent budget.

## 検証

Actual Web status 0 fixtures verify ResumeSlice and Abort provenance, state counters, one-command handoff, cleanup, import call counts, source-policy, normal compile, and full gates.

- actual WASI runtime: Resume evidence 511, Abort evidence 127, aggregate evidence 638; Begin/Run/End calls 2/0/0
- Web source-policy and dedicated normal-mode test-helper isolation passed
- native and wasm32 workspace checks passed
- release CLI build and release trunk build passed
- post-trunk Playground editor CLI JSON passed 13/13
- subagent diff review found no blocker, major, or minor findings
