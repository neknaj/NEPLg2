---
id: ISS-20260604T034207796Z-GUI-AND-TUI-FEATURE-FACADES-STILL-EX-C5A0CA5B
title: "GUI and TUI feature facades still expose backend details during substrate migration"
area: stdlib
status: open
resolved: false
priority: P2
type: architecture
created: 2026-06-04
updated: 2026-06-04
target: "stdlib/features/gui.nepl, stdlib/features/tui.nepl, stdlib/platforms/wasix/tui, stdlib/platforms/gui/terminal"
---

# ISS-20260604T034207796Z-GUI-AND-TUI-FEATURE-FACADES-STILL-EX-C5A0CA5B: GUI and TUI feature facades still expose backend details during substrate migration

## 概要

Subagent audit found features/gui importing platform terminal pieces and features/tui directly exposing platforms/wasix/tui. This conflicts with the requested GUI/TUI common substrate redesign and Zenn dependency direction guidance.

## 対象

- `stdlib/features/gui.nepl, stdlib/features/tui.nepl, stdlib/platforms/wasix/tui, stdlib/platforms/gui/terminal`

## 根拠

- 未記入

## 問題

Subagent audit found features/gui importing platform terminal pieces and features/tui directly exposing platforms/wasix/tui. This conflicts with the requested GUI/TUI common substrate redesign and Zenn dependency direction guidance.

## 影響

Application code can depend on raw ANSI/TTY/terminal backend details through feature facades, making later GUI/TUI unification harder and weakening platform portability.

## 修正方針

Split common UI substrate facade from backend-specific terminal compatibility facade, mark old TUI path as compat/deprecated, and keep raw ANSI/TTY APIs under platforms only.

## 検証

Add import DAG/source policy tests that features/gui does not expose raw ANSI/TTY APIs and that features/tui compatibility remains bounded.
