---
id: ISS-20260430T025134838Z-WEB-DIST-TS-LANGUAGE-ANALYSIS-DROPS--D70C7D62
title: "web dist_ts language-analysis drops diagnostic stable codes"
area: web
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-30
updated: 2026-04-30
target: "web/src/editor-core/language-analysis.ts, web/dist_ts/editor-core/language-analysis.js, nodesrc/test_editor_diagnostic_code_contract.js"
---

# ISS-20260430T025134838Z-WEB-DIST-TS-LANGUAGE-ANALYSIS-DROPS--D70C7D62: web dist_ts language-analysis drops diagnostic stable codes

## 概要

nodesrc/test_editor_diagnostic_code_contract.js fails because web/dist_ts/editor-core/language-analysis.js does not preserve diagnostic code/code_message from analysis snapshots, while web/src/editor-core/language-analysis.ts already maps them. The generated JS artifact is stale relative to the TypeScript source or the build/test flow does not ensure dist_ts is refreshed before the policy test.

## 対象

- `web/src/editor-core/language-analysis.ts, web/dist_ts/editor-core/language-analysis.js, nodesrc/test_editor_diagnostic_code_contract.js`

## 根拠

- `node nodesrc/run_source_policy_regressions.js` は BitSet 関連 policy を含む前半を通過した後、`nodesrc/test_editor_diagnostic_code_contract.js` で `undefined == 'resolve.identifier.undefined'` により失敗した。
- `node nodesrc/test_editor_diagnostic_code_contract.js` 単体でも同じ失敗を再現した。
- `web/src/editor-core/language-analysis.ts` の `collectDiagnostics` は `code: optionalString(item?.code)` と `codeMessage: optionalString(item?.code_message)` を設定している一方、`web/dist_ts/editor-core/language-analysis.js` の生成済み artifact は同 mapping を含んでいない。

## 問題

nodesrc/test_editor_diagnostic_code_contract.js fails because web/dist_ts/editor-core/language-analysis.js does not preserve diagnostic code/code_message from analysis snapshots, while web/src/editor-core/language-analysis.ts already maps them. The generated JS artifact is stale relative to the TypeScript source or the build/test flow does not ensure dist_ts is refreshed before the policy test.

## 影響

Source policy regressions cannot pass from a clean checkout even though the TypeScript source contains the intended diagnostic-code mapping. Editor payloads served from the stale dist_ts artifact can lose stable diagnostic IDs, violating the diagnostics redesign policy.

## 修正方針

Regenerate or stop committing stale dist_ts artifacts, then make the policy/build flow enforce that generated language-analysis.js matches language-analysis.ts before diagnostic code contract tests run.

## 検証

Run npm --prefix web run build:ts, node nodesrc/test_editor_diagnostic_code_contract.js, and node nodesrc/run_source_policy_regressions.js from a clean checkout.

確認済み:

- `node nodesrc/test_editor_diagnostic_code_contract.js`: passed
- `node nodesrc/run_source_policy_regressions.js`: passed

## 修正内容

- `nodesrc/test_editor_diagnostic_code_contract.js` が `web/dist_ts/editor-core/language-analysis.js` を import する前に `npm --prefix web run build:ts` を実行するようにした。
- `web/dist_ts` は git 管理外の生成物なので、stale なローカル artifact を前提にせず、policy test 自体が TypeScript source から dist_ts を更新してから stable diagnostic code / codeMessage contract を検証する。
- Windows では `cmd.exe /d /s /c` 経由、その他では `npm --prefix web run build:ts` を直接実行し、build 失敗時は diagnostic contract test も失敗するようにした。
