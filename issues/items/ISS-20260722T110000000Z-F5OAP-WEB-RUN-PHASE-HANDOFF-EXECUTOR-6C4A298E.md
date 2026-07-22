---
id: ISS-20260722T110000000Z-F5OAP-WEB-RUN-PHASE-HANDOFF-EXECUTOR-6C4A298E
title: "F5oap Web Run phase handoff executor"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-22
updated: 2026-07-22
target: stdlib/platforms/gui/web/font_registered_run_phase_handoff.nepl
---

# F5oap Web Run phase handoff executor

## 概要

F5oal typed Run ownerをactual Web executorへ渡す境界とF5oai success phase classifierは接続済みだが、任意長driverがRunごとに再利用できる単一handoff APIがない。

## 修正方針

F5oal Run ownerを一度だけ既存phase executorへmoveし、成功だけを既存F5oai classifierへ渡す。失敗は既存owner-bearing Run execution errorをそのまま返す。command、cursor、state、completion、spent retry budgetは再構築しない。

## 検証

production two-Run fixtureのRun2をhandoff API経由でactual実行し、payload offset 15 / count 1、Begin 1回 / Run 2回 / End 0回を維持した。owner enumを`Result`へnestせず、Continue / Yield / Completed / Failedのconcrete top-level variantへ移してResource authorityを閉じる。任意長反復budgetとqueue/timerは後続とする。
