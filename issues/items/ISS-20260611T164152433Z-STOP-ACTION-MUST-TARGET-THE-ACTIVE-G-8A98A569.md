---
id: ISS-20260611T164152433Z-STOP-ACTION-MUST-TARGET-THE-ACTIVE-G-8A98A569
title: "Stop action must target the active GUI process owner instead of only focused terminal"
area: tools
status: fixed
resolved: true
priority: P0
type: bug
created: 2026-06-11
updated: 2026-06-11
target: "web/src/main.ts; web/src/workspace/panel-manager.ts; web/src/terminal/shell.ts"
---

# ISS-20260611T164152433Z-STOP-ACTION-MUST-TARGET-THE-ACTIVE-G-8A98A569: Stop action must target the active GUI process owner instead of only focused terminal

## 概要

Toolbar Stop uses only the focused terminal runtime. After focus moves to an editor, GUI window, or another terminal, the actually running GUI process may not stop or the wrong process may be targeted.

## 対象

- `web/src/main.ts; web/src/workspace/panel-manager.ts; web/src/terminal/shell.ts`

## 根拠

- 未記入

## 問題

Toolbar Stop uses only the focused terminal runtime. After focus moves to an editor, GUI window, or another terminal, the actually running GUI process may not stop or the wrong process may be targeted.

## 影響

Runaway GUI apps are difficult to stop reliably, and window close / toolbar stop follow different ownership models.

## 修正方針

Track active process owners in PanelManager or Shell registry. Stop should prefer the focused running terminal, otherwise interrupt the active GUI/process owner associated with visible host windows.

## 検証

Start a GUI app, focus editor/GUI window/another terminal, press Stop, and confirm the correct worker and windows terminate.

## 2026-06-12 Agent2 修正

`PlaygroundPanelManager.stopActiveProcess` を追加した。Stop は focused terminal が実行中ならそれを優先し、focused terminal が無い、または実行中でない場合は実行中 terminal runtime を探索して interrupt する。

toolbar の Stop button は `getFocusedTerminalRuntime` を直接呼ばず、`panelManager.stopActiveProcess` を通すようにした。これにより editor / GUI window / 別 terminal に focus が移った状態でも、実行中 process owner を止められる。

検証:

- `npm --prefix web run build:ts`
- `node nodesrc/test_web_gui_floating_window_source.js`
- `node nodesrc/playground_shell_worker_test_runner.js`
