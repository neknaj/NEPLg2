---
id: ISS-20260722T160000000Z-F5OAU-WEB-BUDGETED-RUN-READY-REENTRY-3C91A7E2
title: "F5oau Web budgeted Run Ready reentry"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-22
updated: 2026-07-22
target: stdlib/platforms/gui/web/font_registered_budgeted_run_driver_test.nepl
---

# F5oau Web budgeted Run Ready reentry

two-Run cursorの最初の未実行Runをfresh ownerとしてdriverへ渡し、1回目の`Ready`が保持するexact Run ownerと残budgetを再発行せず2回目へ渡す。actual Runを各一度だけ実行し、budgetが2/2から1/1、0/0へ単調減少してEndFrameへ到達することを検証する。

actual runtimeはevidence 7、Web Begin/Run/End 1/2/0で通過した。subagent reviewはBlocker/Majorなしで、コメントと実装のdescriptor evidence不一致を修正した。
