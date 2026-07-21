---
id: ISS-20260722T010000000Z-F5OAF-WEB-TERMINAL-COMPLETION-7E62A4D1
title: "F5oaf Web registered terminal completion"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-22
updated: 2026-07-22
target: stdlib/platforms/gui/web/font_registered_terminal_completion.nepl
---

# F5oaf Web registered terminal completion

## 概要

F5oaeはactual EndFrame completionとexact cursorを保持するが、terminal cursor stepと明示的Completedへ閉じていなかった。

## 修正方針

F5oae successだけを一度消費し、actual completionがCompletedであることを検査する。同じownerに保持されたEndFrame stepをF5nzt共通transitionへ一度渡し、得たterminal partsをF5nzvへ一度渡す。provenanceとspent budgetを保持し、schedule、request、session、budgetを再構築しない。

## 検証

actual Web full-chain runtimeはevidence 127、Begin/Run/End各1回で通過し、actual completionとterminal completionのCompleted、state evidence 31、provenanceを確認した。source policyはF5nzt/F5nzv順序と禁止されたauthority再構築がないことを固定する。
