---
id: ISS-20260722T080000000Z-F5OAL-WEB-NEXT-COMMAND-PHASE-7084E136
title: "F5oal Web next command phase classifier"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-22
updated: 2026-07-22
target: stdlib/platforms/gui/web/font_registered_run_next_command.nepl
---

# F5oal Web next command phase classifier

F5oak neutral ownerをBeginFrame、Run、EndFrame、Completedへ全域分類するtyped境界を追加する。raw neutral partsはprivateのまま、exact stepとprovenance、spent budget、stateをvariant ownerへ移す。分類中はcursor advance、resume、host effectを行わない。actual post-Run fixtureではEndFrameを検査し、Runはmulti-Run後続へ残す。
