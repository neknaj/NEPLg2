---
id: ISS-20260611T164152135Z-CLOSING-TERMINAL-PANES-MUST-DISPOSE--EF2A307C
title: "Closing terminal panes must dispose Shell workers GUI windows and input listeners"
area: tools
status: fixed
resolved: true
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

## 2026-06-12 Agent2 修正

`Shell.dispose` を追加し、active worker interrupt、GUI window / timer cleanup、persistent compiler worker termination、GUI input listener unregister をまとめて行うようにした。

`CanvasTerminal.dispose` は blink interval だけでなく `shell.dispose` を呼ぶ。`registerGuiWebInputEventListener` に対応する `unregisterGuiWebInputEventListener` を追加し、terminal pane close / workspace redraw で古い Shell listener が残らないようにした。

検証:

- `npm --prefix web run build:ts`
- `node nodesrc/test_web_gui_input_bridge.js`
- `node nodesrc/test_web_gui_shared_event_queue.js`
- `node nodesrc/test_web_gui_runtime_bridge.js`
- `node nodesrc/test_web_gui_floating_window_source.js`
- `node nodesrc/playground_shell_worker_test_runner.js`
