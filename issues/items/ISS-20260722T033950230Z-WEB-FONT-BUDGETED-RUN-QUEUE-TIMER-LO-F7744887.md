---
id: ISS-20260722T033950230Z-WEB-FONT-BUDGETED-RUN-QUEUE-TIMER-LO-F7744887
title: "Web font budgeted Run queue timer loop boundary"
area: stdlib/gui
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-22
updated: 2026-07-22
target: stdlib/platforms/gui/web/font_registered_budgeted_run_loop.nepl
---

# ISS-20260722T033950230Z-WEB-FONT-BUDGETED-RUN-QUEUE-TIMER-LO-F7744887: Web font budgeted Run queue timer loop boundary

## 概要

Budgeted Run Ready and Suspended outcomes are not connected to the formal Web input queue and one-shot timer wake boundary.

## 対象

- `stdlib/platforms/gui/web/font_registered_budgeted_run_loop.nepl`

## 根拠

- `Ready` は消費済み `BudgetOwner` を変更せずqueue itemへ移す。
- `Suspended` はformal Web one-shot timer requestがhostに受理された後だけpendingになる。
- wakeは`GuiWebEvent`のwindow、timer id、非負tickを照合し、一致時だけ保持済みslice limitからresumeする。

## 問題

Budgeted Run Ready and Suspended outcomes are not connected to the formal Web input queue and one-shot timer wake boundary.

## 影響

Callers can accidentally reissue spent budget or resume a suspended owner without timer evidence.

## 修正方針

Add owner-preserving Ready queue and Suspended timer pending transitions; resume only on matching GuiEvent::Timer using the retained slice limit.

## 検証

- focused Web runtime contractでrequest値とmatching wake後のtotal/slice残量を検査する。
- `trunk build` と Playground editor JSON 13/13 を通す。
