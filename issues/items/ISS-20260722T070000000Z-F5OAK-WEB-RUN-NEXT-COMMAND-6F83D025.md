---
id: ISS-20260722T070000000Z-F5OAK-WEB-RUN-NEXT-COMMAND-6F83D025
title: "F5oak Web Run next command connector"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-22
updated: 2026-07-22
target: stdlib/platforms/gui/web/font_registered_run_next_command.nepl
---

# F5oak Web Run next command connector

## 概要

F5oaiのContinue/Yield ownerから、次commandを分類・実行せずにcursorを一度だけ進める中立なproduction境界がなかった。

## 修正方針

ContinueとYieldを別のconsuming APIで受ける。Yieldだけsliceを一度resetし、current Run stepを閉じてcursorを一度進める。provenance、spent budget、state、exact next stepをsuccess/errorのownerへ保存し、host effectには進まない。

## 検証

source contract、通常import compile、actual Continue/Yield runtimeでownership、slice state、次EndFrame、host call非増加を検査する。任意長反復は後続checkpointである。
