---
id: ISS-20260604T034207429Z-GUI-EXAMPLES-DUPLICATE-FRAME-BUTTON--57AC9031
title: "GUI examples duplicate frame button and event loop boilerplate instead of sharing typed helpers"
area: examples
status: open
resolved: false
priority: P2
type: maintenance
created: 2026-06-04
updated: 2026-06-04
target: "examples/gui_*.nepl, stdlib/platforms/gui/web"
---

# ISS-20260604T034207429Z-GUI-EXAMPLES-DUPLICATE-FRAME-BUTTON--57AC9031: GUI examples duplicate frame button and event loop boilerplate instead of sharing typed helpers

## 概要

Subagent audit found repeated present_frame, present_button, event_loop, action decode, and stdout frame emission code across GUI examples. This conflicts with Zenn zero-cost abstraction, DAG, and responsibility-splitting guidance.

## 対象

- `examples/gui_*.nepl, stdlib/platforms/gui/web`

## 根拠

- 未記入

## 問題

Subagent audit found repeated present_frame, present_button, event_loop, action decode, and stdout frame emission code across GUI examples. This conflicts with Zenn zero-cost abstraction, DAG, and responsibility-splitting guidance.

## 影響

Each example can drift independently in protocol encoding, hit target handling, and result propagation, making future host ABI changes expensive and error-prone.

## 修正方針

Move repeated checked frame/button/event-loop helpers into a shared example support module or platform GUI helper layer, with APIs that remain typed and do not simulate examples in TS.

## 検証

Add helper doctests, source policy for no TS simulation, and focused --once regression tests for each GUI example.
