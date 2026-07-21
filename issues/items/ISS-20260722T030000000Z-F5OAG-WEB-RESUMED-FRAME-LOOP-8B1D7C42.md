---
id: ISS-20260722T030000000Z-F5OAG-WEB-RESUMED-FRAME-LOOP-8B1D7C42
title: "F5oag Web registered resumed frame loop"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-22
updated: 2026-07-22
target: stdlib/platforms/gui/web/font_registered_resumed_web_loop.nepl
---

# F5oag Web registered resumed frame loop

## 概要

F5oab〜F5oafは個別境界として接続済みだったが、F5oaaの未resume Yieldからactual Run、EndFrame、terminal completionまでを一つのproduction呼出しで駆動する境界がなかった。

## 修正方針

concrete Yield ownerを固定ResumeSliceへ一度渡し、next command、Run execution、EndFrame command、EndFrame execution、terminal completionを順に各一度実行する。各失敗はlower ownerを保持し、全variantにconsuming cleanupを設ける。schedule、session、budgetは再構築しない。

## 検証

actual Web runtimeでsuccessとRun failureを同一WASM内で実行し、evidence 510、Begin/Run/End 2/2/1、provenance 1+2+4、spent Exhausted、command count 2、pixel count 16、terminal Completed、drain Ended、top-level failure cleanupを検査した。このfixed chainは任意長loopの完成ではない。
