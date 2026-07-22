---
id: ISS-20260722T140000000Z-F5OAS-WEB-BUDGETED-RUN-DRIVER-STATE-MACHINE-5E8C31A4
title: "F5oas Web budgeted Run driver state machine"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-22
updated: 2026-07-22
target: stdlib/platforms/gui/web/font_registered_budgeted_run_driver.nepl
---

# F5oas Web budgeted Run driver state machine

F5oar budget authorityを各actual Runの前に一度だけ消費し、F5oap handoffのContinue / Yieldをnext-commandへ接続する。次もRunなら同じrun ownerと残budgetを`Ready`として返し、slice exhaustionはrun ownerと専用exhausted budgetを`Suspended`として返す。直接再帰によるcompiler固定点コストを避け、callerが同じstep APIを反復できるresumable state machineにする。

actual runtimeはreplay Runを`ExecutionFailed`として拒否し、消費済みbudget 0/0をerrorとともに保持して全ownerをcleanupする経路をevidence 48、Web Begin/Run/End 1/1/0で確認した。fresh pre-Run owner producerが接続された後、Continue/YieldからReadyへの成功反復、TotalExhausted、Suspendedのactual統合試験を追加する。
