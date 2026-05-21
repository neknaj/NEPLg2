---
id: ISS-20260521T224131240Z-RESOURCE-RESPONSIBILITY-MONITOR-MISS-B6468B0C
title: "Resource responsibility monitor misses drop_call_identity module"
area: tools
status: open
resolved: false
priority: P2
type: test
created: 2026-05-21
updated: 2026-05-21
target: "nodesrc/test_resource_checker_responsibility.js, nepl-core/src/resource/drop_call_identity.rs"
---

# ISS-20260521T224131240Z-RESOURCE-RESPONSIBILITY-MONITOR-MISS-B6468B0C: Resource responsibility monitor misses drop_call_identity module

## 概要

nodesrc/test_resource_checker_responsibility.js fails because the newly added nepl-core/src/resource/drop_call_identity.rs module is not included in the monitored resource responsibility line-limit set.

## 対象

- `nodesrc/test_resource_checker_responsibility.js, nepl-core/src/resource/drop_call_identity.rs`

## 根拠

- 2026-05-22 Agent 1 の optional broader monitor 実行で `node nodesrc/test_resource_checker_responsibility.js` が `drop_call_identity.rs must be monitored by resource responsibility line limits` として失敗した。
- `drop_call_identity.rs` は Drop trait call と Resource IR drop proof の identity bridge を担う静的検査 core module であり、責務監視対象から漏れると大規模静的検査修正の module size / ownership boundary 監査が弱くなる。

## 問題

nodesrc/test_resource_checker_responsibility.js fails because the newly added nepl-core/src/resource/drop_call_identity.rs module is not included in the monitored resource responsibility line-limit set.

## 影響

Resource checker responsibility monitoring can fail on current main and no longer covers all Resource IR modules, weakening regression visibility for static-check refactors.

## 修正方針

Add drop_call_identity.rs to the appropriate responsibility monitor set and verify the monitor still enforces module size and ownership boundaries without relaxing limits.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js and node nodesrc/issues.js check --dir issues.
