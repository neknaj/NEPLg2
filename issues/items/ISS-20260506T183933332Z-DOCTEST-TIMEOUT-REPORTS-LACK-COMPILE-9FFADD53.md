---
id: ISS-20260506T183933332Z-DOCTEST-TIMEOUT-REPORTS-LACK-COMPILE-9FFADD53
title: "doctest timeout reports lack compile/run phase timing"
area: test
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-06
target: "nodesrc/run_test.js, nodesrc/tests.js, nodesrc/tests_wasm_worker.js"
---

# ISS-20260506T183933332Z-DOCTEST-TIMEOUT-REPORTS-LACK-COMPILE-9FFADD53: doctest timeout reports lack compile/run phase timing

## 概要

nodesrc doctest reports only total duration_ms for completed cases and timeout results do not record whether the worker was compiling or running. KP doctest investigation therefore misclassified a compile-time dominated case as a float runtime timeout.

## 対象

- `nodesrc/run_test.js, nodesrc/tests.js, nodesrc/tests_wasm_worker.js`

## 根拠

- `tests/stdlib/kp.n.md::doctest#1` は小さい整数 scanner case だが、`run_doctest.js` の計測で total 約 47.5 秒のうち compile が約 47.5 秒、run が約 14ms だった。
- 既存 JSON は `duration_ms` しか持たず、完了 case でも compile / run のどちらが重いかを machine-readable に判定できなかった。
- `nodesrc/tests.js` の timeout result は worker の最終 phase を持たないため、60 秒 timeout が compiler wasm の同期 compile 中なのか、生成 wasm の WASI run 中なのかを判別できなかった。
- focused suite が既定 flush 間隔 10 件より小さい場合、外側の command timeout で終了すると最後の completed result が partial JSON に残らないことがあった。

## 問題

nodesrc doctest reports only total duration_ms for completed cases and timeout results do not record whether the worker was compiling or running. KP doctest investigation therefore misclassified a compile-time dominated case as a float runtime timeout.

## 影響

Agents and CI can chase the wrong subsystem, hide compiler performance regressions behind runtime wording, and lose useful partial progress for small focused suites.

## 修正方針

Record structured timing for compile and run phases, propagate worker phase progress to timeout results, and flush small-suite partial JSON often enough to preserve the latest completed result.

## 検証

Run source policy tests for doctest progress and focused KP doctest commands to confirm timing metadata and timeout phase fields are present.

## 対応

- `nodesrc/run_test.js` の result に `timing.load_ms` / `timing.compile_ms` / `timing.run_ms` / `timing.total_ms` を追加した。
- `runSingle` に phase progress callback を追加し、worker runner が `load` / `compile` / `run` の start/end を親へ通知できるようにした。
- `nodesrc/tests.js` は worker からの progress を実行中 case に保持し、timeout result の `phase` と `timeout.last_phase` / `last_event` / `elapsed_ms` に反映する。
- focused suite の expected result 数が既定 flush 間隔より小さい場合は 1 件ごとに partial JSON を flush し、command-level timeout でも直近結果を失いにくくした。
- `nodesrc/test_run_test_timing_metadata.js` と `nodesrc/test_tests_js_partial_progress_policy.js` で timing metadata / phase progress / timeout phase contract を固定した。

## 対応後の確認結果

- `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 1` は passed。JSON 出力は `compile_ms=47510`, `run_ms=14`, `total_ms=47565` で、遅さが runtime ではなく compiler phase にあることを示した。
- `NEPL_TEST_CASE_TIMEOUT_MS=2000 node nodesrc/tests.js -i tests/stdlib/kp.n.md --no-tree -o tmp/agent1-kp-timeout-phase.json -j 1 --assert-io` は expected timeout だが、各 top issue の `phase` が `compile` になり、`timeout.last_phase=compile` が JSON に残った。
