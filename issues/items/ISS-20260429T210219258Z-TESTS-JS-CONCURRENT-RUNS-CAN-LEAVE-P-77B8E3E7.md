---
id: ISS-20260429T210219258Z-TESTS-JS-CONCURRENT-RUNS-CAN-LEAVE-P-77B8E3E7
title: "tests.js concurrent runs can leave partial result files without diagnostics"
area: test
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-29
updated: 2026-04-30
target: nodesrc/tests.js
---

# ISS-20260429T210219258Z-TESTS-JS-CONCURRENT-RUNS-CAN-LEAVE-P-77B8E3E7: tests.js concurrent runs can leave partial result files without diagnostics

## 概要

Running multiple nodesrc/tests.js processes at the same time can exit with code 1 after writing only the initial partial JSON (partial_reason: started, completed_results: 0) and no actionable top_issues. This was reproduced while BTreeMap/BTreeSet stdlib tests were executed in parallel with other tests; rerunning the same command alone passed.

## 対象

- `nodesrc/tests.js`

## 根拠

- BTreeMap/BTreeSet focused test を別プロセスと並行実行した際、`partial_reason: started` / `completed_results: 0` の JSON だけを残して exit code 1 になった。
- その JSON には `top_issues` がなく、worker failure / harness failure / actual test failure の切り分けができなかった。
- 同じ test command を単独で再実行すると pass したため、stdlib regression ではなく harness concurrency / progress output の問題として分離した。

## 問題

Running multiple nodesrc/tests.js processes at the same time can exit with code 1 after writing only the initial partial JSON (partial_reason: started, completed_results: 0) and no actionable top_issues. This was reproduced while BTreeMap/BTreeSet stdlib tests were executed in parallel with other tests; rerunning the same command alone passed.

## 影響

Agents may treat a harness/process contention failure as a stdlib regression, and CI or local scripted verification can lose diagnostic context if tests.js instances share global state or compete for dist/compiler resources.

## 修正方針

Audit tests.js worker/result lifecycle and any shared temp/dist/compiler state so each process records child-process failures explicitly. Either support concurrent invocations or serialize/lock the shared resources with a clear diagnostic.

## 対応

- `nodesrc/tests.js` の output JSON 書き込みを temp file + rename の atomic write に変更した。final path へ直接書き込む途中状態を残さない。
- wasm runner / legacy runner の case 完了時に `recordProgress` を呼び、`partial_reason: started` のまま進捗 0 で残る時間をなくした。
- fail / error result は flush interval を待たずに partial JSON へ即時反映するようにした。
- wasm partial progress は `applyDoctestExpectations` 適用後の status を記録し、ret / stdout / stderr mismatch も partial `top_issues` に出るようにした。
- partial JSON に `top_issues` を追加した。
- harness 内部例外は `nodesrc/tests/internal-error` の error result として JSON に記録し、`partial_reason: error` を出すようにした。
- timeout 処理で意図的に terminate した worker の exit を、別の worker crash として誤検出しないようにした。
- `nodesrc/test_tests_js_partial_progress_policy.js` を追加し、atomic write / partial top_issues / expectation-checked wasm progress / internal error JSON 化を source policy で固定した。
- `nodesrc/test_tests_js_concurrent_runs_complete_json.js` を追加し、2 つの `tests.js` を同時実行して両方が complete final JSON を生成することを検証した。

## 検証

Add a regression script that launches two independent tests.js invocations concurrently against small fixtures and asserts both produce complete JSON with resolved_dist_dirs and final summaries.

- `node nodesrc/test_tests_js_partial_progress_policy.js`: passed
- `node nodesrc/test_tests_js_concurrent_runs_complete_json.js`: passed
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed
