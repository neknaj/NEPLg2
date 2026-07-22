---
id: ISS-20260722T120000000Z-F5OAQ-WEB-COMMAND-DRIVER-BUDGET-AUTHORITY-3E91B20C
title: "F5oaq Web command driver budget authority"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-22
updated: 2026-07-22
target: stdlib/platforms/gui/web/font_registered_command_driver_budget.nepl
---

# F5oaq Web command driver budget authority

任意長command driverがtotal/slice budgetをraw整数として再発行せず輸送できるmodule-private typed ownerを追加した。start時にpositive total/slice limitを検査し、各actionで両残量を一度だけ減らす。slice exhaustedだけが保持済みslice limitからresumeでき、total exhaustedは再発行不能とする。driver本体とqueue/timerは後続である。
