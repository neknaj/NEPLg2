---
id: ISS-20260604T034207064Z-BREAKOUT-GUI-EXAMPLE-USES-TIMEOUT-NO-EA0E4163
title: "Breakout GUI example uses timeout None as animation tick instead of GuiEvent Timer"
area: examples
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-06-04
updated: 2026-06-05
target: "examples/gui_breakout.nepl, stdlib/platforms/gui/web/input.nepl, web/src/runtime/worker.ts, web/src/gui-preview/shared-event-queue.ts, web/src/terminal/shell.ts"
---

# ISS-20260604T034207064Z-BREAKOUT-GUI-EXAMPLE-USES-TIMEOUT-NO-EA0E4163: Breakout GUI example uses timeout None as animation tick instead of GuiEvent Timer

## 概要

Subagent audit found Breakout driving animation ticks through timeout fallback and Option::None while platform input notes timer/lifecycle support as future work. This collapses input absence and tick into one state, conflicting with enum/match event modeling.

## 対象

- `examples/gui_breakout.nepl, stdlib/platforms/gui/web/input.nepl, stdlib/std/gui/runtime.nepl`

## 根拠

- 未記入

## 問題

Subagent audit found Breakout driving animation ticks through timeout fallback and Option::None while platform input notes timer/lifecycle support as future work. This collapses input absence and tick into one state, conflicting with enum/match event modeling.

## 影響

Pause, animation cadence, busy-loop prevention, and event ordering cannot be specified precisely, and the example bypasses the scheduler/timeslice design that GUI needs.

## 修正方針

Connect GuiEvent::Timer through Web input/runtime, update Breakout to match timer events explicitly, and remove the timeout fallback from the example loop.

## 検証

- `npm --prefix web run build:ts`
- `node nodesrc/test_web_gui_input_bridge.js`
- `node nodesrc/test_web_gui_shared_event_queue.js`
- `node nodesrc/test_web_gui_stdout_protocol.js`
- `node nodesrc/tests.js -i examples/gui_breakout.nepl --no-tree -o tmp/gui-breakout-timer-event.json -j 1 --dist web/dist --assert-io`

## 修正内容

- Web input queue に raw kind 6 の timer record を追加した。
- Worker の `nepl_gui_web` host import に `last_event_timer_id` / `last_event_timer_tick` を追加した。
- `platforms/gui/web/input.nepl` は timer record を `GuiEvent::Timer` へ正規化する。
- stdout protocol の animation timer request は window id と timer id を持つ typed command になった。
- Shell は active window に対して browser timer を管理し、tick を typed input queue へ戻す。
- Breakout は `Option::None` timeout では tick せず、`gui_web_event_timer` を受けた時だけ animation を進める。
