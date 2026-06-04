---
id: ISS-20260604T034207064Z-BREAKOUT-GUI-EXAMPLE-USES-TIMEOUT-NO-EA0E4163
title: "Breakout GUI example uses timeout None as animation tick instead of GuiEvent Timer"
area: examples
status: open
resolved: false
priority: P1
type: architecture
created: 2026-06-04
updated: 2026-06-04
target: "examples/gui_breakout.nepl, stdlib/platforms/gui/web/input.nepl, stdlib/std/gui/runtime.nepl"
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

Connect GuiEvent::Timer through Web input/runtime, update Breakout to match timer events explicitly, and document timeout fallback only as a temporary compatibility path.

## 検証

Add timer decode, tick/action separation, pause behavior, busy-loop prevention, and scheduler/timeslice regular tests.
