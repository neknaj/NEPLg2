---
id: ISS-20260611T143313507Z-PAGES-TEST-RESULTS-NEED-STRUCTURED-C-793A6635
title: "Pages test results need structured current and completed artifacts"
area: ci
status: verified
resolved: true
priority: P1
type: bug
created: 2026-06-11
updated: 2026-06-11
target: "web/tests.html,.github/workflows/ci.yml"
---

# ISS-20260611T143313507Z-PAGES-TEST-RESULTS-NEED-STRUCTURED-C-793A6635: Pages test results need structured current and completed artifacts

## 概要

tests.html showed flat JSON artifacts and pending Pages deployments hid the latest completed test results while the current run was still pending.

## 対象

- `web/tests.html,.github/workflows/ci.yml`

## 根拠

- 未記入

## 問題

tests.html showed flat JSON artifacts and pending Pages deployments hid the latest completed test results while the current run was still pending.

## 影響

Users could see raw JSON/ANSI diagnostic text or pending-only status instead of readable test results and pass rates.

## 修正方針

Publish Rust and selfhost compiler-check JSON artifacts separately, preserve last-completed artifacts for pending Pages deployments, and render hierarchical pass-rate summaries in tests.html.

## 検証

node nodesrc/test_pages_ci_metrics_contract.js; node nodesrc/test_ci_timeout_policy.js; node nodesrc/run_source_policy_regressions.js --warn-only; inline web/tests.html script syntax check; GitHub Pages deployment artifact includes current and last-completed test JSON.

## 対応結果

- `.github/workflows/ci.yml` に `selfhost-doctest` matrix を追加し、Rust compiler 実装の doctest JSON と selfhost compiler check JSON を別 artifact として Pages に publish するようにした。
- pending Pages artifact は `tests/last-completed/*.json` を前回公開済み Pages から退避し、current commit の結果が未完了であることと表示中の直近完了済み結果を分けて扱えるようにした。
- `web/tests.html` は ANSI diagnostic を HTML 表示へ変換し、`suite -> implementation -> method -> path` の階層で通過率を表示するようにした。
- `nodesrc/run_selfhost_doctest_check.js` は Rust JSON を複製せず、NEPL harness 内で selfhost compiler pipeline を呼ぶ compiler check JSON を生成する。runtime assertions ではないため `runtime_assertions=false` として区別する。
- selfhost compiler check が timeout した場合は `nodesrc/complete_selfhost_doctest_artifact.js` が timeout marker を `neplg2-selfhost-doctest/v1` JSON の synthetic timeout result に変換し、Pages 上で artifact 欠落を成功のように見せないようにした。
