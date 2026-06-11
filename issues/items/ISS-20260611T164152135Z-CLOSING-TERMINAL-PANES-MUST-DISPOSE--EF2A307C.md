---
id: ISS-20260611T164152135Z-CLOSING-TERMINAL-PANES-MUST-DISPOSE--EF2A307C
title: "Closing terminal panes must dispose Shell workers GUI windows and input listeners"
area: tools
status: open
resolved: false
priority: P0
type: bug
created: 2026-06-11
updated: 2026-06-11
target: "web/src/terminal/terminal.ts; web/src/terminal/shell.ts; web/src/workspace/panel-manager.ts; web/src/gui-preview/input-bridge.ts"
---

# ISS-20260611T164152135Z-CLOSING-TERMINAL-PANES-MUST-DISPOSE--EF2A307C: Closing terminal panes must dispose Shell workers GUI windows and input listeners

## 概要

CanvasTerminal.dispose only clears terminal blink state. Removed terminal runtimes do not dispose Shell active workers, GUI runtime windows/timers, or registered GUI input listeners.

## 対象

- `web/src/terminal/terminal.ts; web/src/terminal/shell.ts; web/src/workspace/panel-manager.ts; web/src/gui-preview/input-bridge.ts`

## 根拠

- 未記入

## 問題

CanvasTerminal.dispose only clears terminal blink state. Removed terminal runtimes do not dispose Shell active workers, GUI runtime windows/timers, or registered GUI input listeners.

## 影響

Closing a terminal can leave GUI apps running, windows orphaned, and old Shell listeners receiving future GUI input.

## 修正方針

Add Shell.dispose that interrupts active work, closes GUI windows, clears timers, unregisters input listeners, and releases compiler/runtime workers. Terminal/panel disposal must call it.

## 検証

Run GUI app, close terminal pane, and assert worker termination, floating window removal, timer cleanup, and listener count stability.
