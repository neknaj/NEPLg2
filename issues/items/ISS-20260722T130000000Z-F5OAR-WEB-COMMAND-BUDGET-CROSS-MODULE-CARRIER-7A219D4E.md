---
id: ISS-20260722T130000000Z-F5OAR-WEB-COMMAND-BUDGET-CROSS-MODULE-CARRIER-7A219D4E
title: "F5oar Web command budget cross-module carrier"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-22
updated: 2026-07-22
target: stdlib/platforms/gui/web/font_registered_command_driver_budget.nepl
---

# F5oar Web command budget cross-module carrier

F5oaq budget ownerはraw constructor再発行を防ぐため型全体をmodule-privateにしたが、後続driver moduleのowner fieldとして型名を保持できない。owner nominalをpublicにし、active / slice-exhaustedで別々のmodule-private proofを唯一のcanonical stateとして保持させた。外部moduleはownerを型として輸送し、既存proofを線形に再包装できるが、raw値からproofをmintしたりactive proofをslice-exhausted authorityへ変換したりできない。

cross-module runtimeはownerをcarrier fieldへ移し、proofを型推論で取り出して同じactive ownerへ再包装した後も残量が増えないことを確認する。compile-fail fixtureは両proofのraw mintとactive proofからslice-exhausted ownerへのtypestate偽装を`type.overload.no_match`で固定する。focused runtimeはevidence 32で通過した。
