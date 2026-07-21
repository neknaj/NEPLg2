---
id: ISS-20260721T172102186Z-F5OAE-WEB-ENDFRAME-SCHEDULE-REQUEST--01A404F1
title: "F5oae Web EndFrame schedule request execution"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-21
updated: 2026-07-22
target: stdlib/platforms/gui/web/font_registered_end_frame_executor.nepl
---

# ISS-20260721T172102186Z-F5OAE-WEB-ENDFRAME-SCHEDULE-REQUEST--01A404F1: F5oae Web EndFrame schedule request execution

## 概要

F5oad stops at exact EndFrame command authority and does not perform same-record schedule request or Web execution.

## 対象

- `stdlib/platforms/gui/web/font_registered_end_frame_executor.nepl`

## 根拠

- F5oadのtyped ownerだけがpost-Run stateとexact EndFrame cursorを同居保持する。
- F5nzr/F5nzsはrecord projectionとschedule-only authorityを既に提供する。

## 問題

F5oad stops at exact EndFrame command authority and does not perform same-record schedule request or Web execution.

## 影響

The actual registered Web path cannot complete EndFrame or hand its completion to the terminal transition.

## 修正方針

Consume F5oad once, preserve provenance and spent budget, and pass its exact EndFrame owner through F5nzr/F5nzs, a typed same-record host-request bridge, F5nh, and the existing Web executor exactly once.

## 検証

Actual full-chain runtime returns evidence 63 and observes Begin 1 / Run 1 / End 1; relative to the F5oad fixture the new boundary adds only one EndFrame call. It retains typed Completed completion and the EndFrame cursor. Source policy verifies ordering, recovery ownership, and absence of terminal advancement.
