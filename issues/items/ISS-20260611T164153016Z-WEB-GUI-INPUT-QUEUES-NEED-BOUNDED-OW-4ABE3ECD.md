---
id: ISS-20260611T164153016Z-WEB-GUI-INPUT-QUEUES-NEED-BOUNDED-OW-4ABE3ECD
title: "Web GUI input queues need bounded ownership telemetry and stale-event reset"
area: tools
status: open
resolved: false
priority: P1
type: architecture
created: 2026-06-11
updated: 2026-06-11
target: "web/src/gui-preview/input-bridge.ts; web/src/gui-preview/shared-event-queue.ts; web/src/terminal/shell.ts; web/src/gui-preview/window-manager.ts"
---

# ISS-20260611T164153016Z-WEB-GUI-INPUT-QUEUES-NEED-BOUNDED-OW-4ABE3ECD: Web GUI input queues need bounded ownership telemetry and stale-event reset

## 概要

GUI input is stored in both a module-global retained queue and a SharedArrayBuffer queue. The retained queue can accumulate when no JS consumer drains it, while SAB overflow silently drops oldest events without app-visible telemetry.

## 対象

- `web/src/gui-preview/input-bridge.ts; web/src/gui-preview/shared-event-queue.ts; web/src/terminal/shell.ts; web/src/gui-preview/window-manager.ts`

## 根拠

- 未記入

## 問題

GUI input is stored in both a module-global retained queue and a SharedArrayBuffer queue. The retained queue can accumulate when no JS consumer drains it, while SAB overflow silently drops oldest events without app-visible telemetry.

## 影響

Long-running apps can accumulate stale events, future consumers can read previous-run input, and input loss in paint/game apps is silent and hard to diagnose.

## 修正方針

Separate listener fan-out from retained queue ownership, make retained queue bounded or disabled for SAB runs, reset queues on run boundaries, and expose dropped/coalesced event counters to app/debug UI.

## 検証

Tests for massive input without takeInputEvents, run boundary reset, capacity overflow dropped counter, and debug panel visibility.
