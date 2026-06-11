---
id: ISS-20260611T164152722Z-GUI-CLOSE-REQUESTED-LIFECYCLE-MUST-C-9494308F
title: "GUI close requested lifecycle must choose app event or force-stop contract"
area: tools
status: open
resolved: false
priority: P1
type: architecture
created: 2026-06-11
updated: 2026-06-11
target: "web/src/terminal/shell.ts; web/src/gui-preview/window-manager.ts; stdlib/platforms/gui/web/input.nepl"
---

# ISS-20260611T164152722Z-GUI-CLOSE-REQUESTED-LIFECYCLE-MUST-C-9494308F: GUI close requested lifecycle must choose app event or force-stop contract

## 概要

The window manager queues close-requested, but Shell intercepts it before writing to the shared queue and immediately interrupts the process. stdlib exposes close-requested as an event, but apps cannot observe it.

## 対象

- `web/src/terminal/shell.ts; web/src/gui-preview/window-manager.ts; stdlib/platforms/gui/web/input.nepl`

## 根拠

- 未記入

## 問題

The window manager queues close-requested, but Shell intercepts it before writing to the shared queue and immediately interrupts the process. stdlib exposes close-requested as an event, but apps cannot observe it.

## 影響

NEPL GUI apps cannot implement graceful close, confirmation, save, or cleanup behavior. The public event model and actual Web behavior disagree.

## 修正方針

Either deliver close-requested to the app with a timeout/second-close force-stop policy, or document Web close as a stop command and remove it from the app-visible event contract for this backend.

## 検証

Add a GUI example/test that observes WindowEvent::CloseRequested and exits voluntarily, or a contract test proving close is intentionally force-stop only.
