---
id: ISS-20260719T142644710Z-F5OAC-WEB-RESUMED-RUN-EXECUTION-AND--1D6B381A
title: "F5oac Web resumed Run execution and completion rejoin"
area: gui-font
status: fixed
resolved: true
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

- F5nzp schedule-only authority now retains pre-state, updated state, phase, and exact record in one move-only value.
- The host-request adopter returns either a pending request or the intact schedule authority with `GuiError`.
- The Web adapter executes the retained Run through F5ne/F5nh and the existing Web executor exactly once.

## 問題

F5oab retains a typed Run command and retry provenance but has no canonical Web execution path that rejoins the dispatch completion.

## 影響

The actual Web retry success path stops before executing its first Run record.

## 修正方針

Consume the F5oab next-command authority once, project and schedule the Run through F5nzo/F5nzp, create the request from the same formal record, and execute the exact F5nh pending action once with the existing Web executor while retaining provenance, cursor, and owner-bearing failure recovery.

## 検証

Actual Web status 0 fixture verifies Begin once, Run once, End zero, Continue completion 1/16, provenance, and cleanup. Actual Run status -1 verifies Unsupported, pre-Run rollback 0/0, provenance, lower recovery consumption, and cleanup. Lower runtime evidence is 30; Web success/failure evidence is 1038 with calls 2/2/0. Source policy, normal isolation, and full repository gates pass.
