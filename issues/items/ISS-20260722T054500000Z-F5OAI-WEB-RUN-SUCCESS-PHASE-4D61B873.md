---
id: ISS-20260722T054500000Z-F5OAI-WEB-RUN-SUCCESS-PHASE-4D61B873
title: "F5oai Web Run success phase classifier"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-22
updated: 2026-07-22
target: stdlib/platforms/gui/web/font_registered_run_success_phase.nepl
---

# F5oai Web Run success phase classifier

## 概要

F5oadはactual Run completionのYield/CompletedをContinueExpected errorへ変換するため、任意長loopがRun後の全phaseを所有権付きで扱えるproduction boundaryがなかった。

## 修正方針

F5oac successを一度だけparts化し、actual completionでContinue/Yield/Completedへ全域分類する。各variantはprovenance、spent budget、exact Run step、actual completionを保持し、stateを複製しない。分類時にcursor advance、resume、EndFrame遷移、host executionを行わない。

## 検証

actual Run fixtureでContinueとYieldを生成し、Begin/Run各2回、End 0回、variant分類とcleanupを検査する。Completedは正常なRun単体から通常生成されないためproductionで捏造せず、全域source contractで固定する。bounded resumable driverは後続である。
