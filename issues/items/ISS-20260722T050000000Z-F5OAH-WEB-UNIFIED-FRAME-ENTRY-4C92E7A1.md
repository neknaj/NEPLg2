---
id: ISS-20260722T050000000Z-F5OAH-WEB-UNIFIED-FRAME-ENTRY-4C92E7A1
title: "F5oah Web registered unified frame entry"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-22
updated: 2026-07-22
target: stdlib/platforms/gui/web/font_registered_unified_web_frame.nepl
---

# F5oah Web registered unified frame entry

## 概要

F5nzzからF5oagまでの個別境界は接続済みだが、RetryReadyをactual Begin retryへ渡し、成功phaseを分類してYieldだけを固定Run/EndFrame chainへ渡す単一production entryがなかった。

## 修正方針

RetryReadyをF5nzzへ一度渡し、F5oaaでphaseを一度分類する。Continue/Completedは正常ownerとしてcallerへ返し、YieldだけをF5oagへ一度渡す。retry failureとresumed-loop failureはlower authorityを保持し、全variantを閉じられるようにする。

## 検証

actual Web runtimeでBegin/Run/End各1回、terminal Completed、provenance、spent Exhausted、command count 2、pixel count 16、drain Endedを検査する。任意長loop、queue/timer、Continue暗黙resume、再retryは後続である。
