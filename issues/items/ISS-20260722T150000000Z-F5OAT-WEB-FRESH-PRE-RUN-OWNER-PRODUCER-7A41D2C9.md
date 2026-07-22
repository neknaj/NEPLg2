---
id: ISS-20260722T150000000Z-F5OAT-WEB-FRESH-PRE-RUN-OWNER-PRODUCER-7A41D2C9
title: "F5oat Web fresh pre-Run owner producer"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-22
updated: 2026-07-22
target: stdlib/platforms/gui/web/font_registered_fresh_run_owner.nepl
---

# F5oat Web fresh pre-Run owner producer

retry Yield再開後の未実行next-command authorityをborrowed resultで検査し、Runの場合だけprovenance、state、stepを分断せずF5oas driver用Run ownerへ移す。Run以外は元ownerをそのまま返し、実行済みsuccessからRun ownerを再構築するreplay経路をactual runtimeから除去する。

actual runtimeはfresh ownerからRunを一度だけ実行し、EndFrame outcome、evidence 3、Web Begin/Run/End 1/1/0を確認した。NotRun / Completedの実行可能fixtureと、複数fresh RunによるReady再投入、TotalExhausted、Suspendedは後続とする。
