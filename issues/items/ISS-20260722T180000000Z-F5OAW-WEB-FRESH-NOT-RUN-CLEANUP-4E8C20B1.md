---
id: ISS-20260722T180000000Z-F5OAW-WEB-FRESH-NOT-RUN-CLEANUP-4E8C20B1
title: "F5oaw Web fresh NotRun cleanup"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-22
updated: 2026-07-22
target: stdlib/platforms/gui/web/font_registered_fresh_run_owner_test.nepl
---

# F5oaw Web fresh NotRun cleanup

actual fresh Run fixtureのcursor authorityをテストadapter内でEndFrame/Completed出力へ構造的に進め、同じprovenanceを持つnext-command ownerとしてfresh producerへ渡す。producerがRunへ誤分類せず元ownerをNotRunで返し、両出力をconsuming cleanupできることを実行検証する。host Run/EndFrameは呼ばない。

runtimeはEndFrame evidence 3とCompleted evidence 5の合計8、Web Begin/Run/End 2/0/0で通過した。subagent reviewはBlocker/Major/Minorすべて0だった。
