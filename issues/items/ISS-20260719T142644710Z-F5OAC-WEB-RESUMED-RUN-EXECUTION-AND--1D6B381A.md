---
id: ISS-20260719T142644710Z-F5OAC-WEB-RESUMED-RUN-EXECUTION-AND--1D6B381A
title: "F5oac Web resumed Run execution and completion rejoin"
area: gui-font
status: open
resolved: false
priority: P1
type: architecture
created: 2026-07-19
updated: 2026-07-19
target: stdlib/platforms/gui/web/font_registered_run_executor.nepl
---

# ISS-20260719T142644710Z-F5OAC-WEB-RESUMED-RUN-EXECUTION-AND--1D6B381A: F5oac Web resumed Run execution and completion rejoin

## 概要

F5oab retains a typed Run command and retry provenance but has no canonical Web execution path that rejoins the dispatch completion.

## 対象

- `stdlib/platforms/gui/web/font_registered_run_executor.nepl`

## 根拠

- 未記入

## 問題

F5oab retains a typed Run command and retry provenance but has no canonical Web execution path that rejoins the dispatch completion.

## 影響

The actual Web retry success path stops before executing its first Run record.

## 修正方針

Consume the F5oab next-command authority once, project the Run record through F5nzo, submit it through the canonical F5nc combined dispatch, and execute the exact F5nh pending action once with the existing Web executor while retaining provenance and cursor authority.

## 検証

Actual Web status 0 fixture verifies Begin once, Run once, End zero, Continue completion, state counters, provenance, cleanup, source-policy, normal isolation, and full gates.
