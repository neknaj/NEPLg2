---
id: ISS-20260722T061000000Z-F5OAJ-BOUNDED-WEB-RUN-PHASE-DRIVER-5E72C914
title: "F5oaj bounded Web Run phase driver"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-22
updated: 2026-07-22
target: stdlib/platforms/gui/web/font_registered_run_phase_driver.nepl
---

# F5oaj bounded Web Run phase driver

## 概要

F5oaiはRun後phaseを保持できるが、ContinueをEndFrameへ進め、Yieldをreplayなしで一度suspend/resumeするproduction driverがなかった。

## 修正方針

typed action/yield budgetとphase ownerを線形に扱う。Suspended ownerはexact phaseと残budgetを同居保持してopaqueにし、module-private partsだけをresumeが消費する。Yieldの`DeferOnce`だけを内部で`OneRemaining`へ遷移させ、callerからfresh policyを受け取らない。Continueは直接、Yieldは一度resume後に既存EndFrame/terminal chainへ渡す。Completedはterminalを捏造せずtyped stopにする。

## 検証

actual Continue terminalとactual Yield suspend/resumeを同一runtimeで検査し、Begin/Run/End合計2/2/2、evidence 3を期待する。queue/timer、任意長反復、fresh Begin入口は後続である。
