---
id: ISS-20260429T210219258Z-TESTS-JS-CONCURRENT-RUNS-CAN-LEAVE-P-77B8E3E7
title: "tests.js concurrent runs can leave partial result files without diagnostics"
area: test
status: open
resolved: false
priority: P2
type: bug
created: 2026-04-29
updated: 2026-04-29
target: nodesrc/tests.js
---

# ISS-20260429T210219258Z-TESTS-JS-CONCURRENT-RUNS-CAN-LEAVE-P-77B8E3E7: tests.js concurrent runs can leave partial result files without diagnostics

## 概要

Running multiple nodesrc/tests.js processes at the same time can exit with code 1 after writing only the initial partial JSON (partial_reason: started, completed_results: 0) and no actionable top_issues. This was reproduced while BTreeMap/BTreeSet stdlib tests were executed in parallel with other tests; rerunning the same command alone passed.

## 対象

- `nodesrc/tests.js`

## 根拠

- 未記入

## 問題

Running multiple nodesrc/tests.js processes at the same time can exit with code 1 after writing only the initial partial JSON (partial_reason: started, completed_results: 0) and no actionable top_issues. This was reproduced while BTreeMap/BTreeSet stdlib tests were executed in parallel with other tests; rerunning the same command alone passed.

## 影響

Agents may treat a harness/process contention failure as a stdlib regression, and CI or local scripted verification can lose diagnostic context if tests.js instances share global state or compete for dist/compiler resources.

## 修正方針

Audit tests.js worker/result lifecycle and any shared temp/dist/compiler state so each process records child-process failures explicitly. Either support concurrent invocations or serialize/lock the shared resources with a clear diagnostic.

## 検証

Add a regression script that launches two independent tests.js invocations concurrently against small fixtures and asserts both produce complete JSON with resolved_dist_dirs and final summaries.
