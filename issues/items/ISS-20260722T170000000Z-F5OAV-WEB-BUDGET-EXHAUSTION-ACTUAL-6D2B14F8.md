---
id: ISS-20260722T170000000Z-F5OAV-WEB-BUDGET-EXHAUSTION-ACTUAL-6D2B14F8
title: "F5oav Web budget exhaustion actual paths"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-22
updated: 2026-07-22
target: stdlib/platforms/gui/web/font_registered_budgeted_run_driver_test.nepl
---

# F5oav Web budget exhaustion actual paths

two-Run fresh authorityでtotal 1 / slice 1の2回目Runを`TotalExhausted`として実行前に停止する。別authorityではtotal 2 / slice 1の2回目Runを`Suspended`として停止し、保持済みslice limitだけでresumeしてEndFrameへ進める。停止中のRun ownerを再構築せず、host Run回数とbudget単調減少を検証する。

経路別runtimeでTotalExhaustedはevidence 5、Web Begin/Run/End 1/1/0、Suspended-resumeはevidence 11、1/2/0を確認した。subagent reviewはBlocker/Majorなし。合算call counterが経路間の配分を隠すMinorを別runSingleへ分離して修正した。
