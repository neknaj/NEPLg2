---
id: ISS-20260506T173138867Z-RESOURCE-RAW-ADDRESS-LOWERING-EXCEED-B64EB6D1
title: "Resource raw address lowering exceeds split limit after deep-prefix alias fix"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/src/resource/lower_raw_address.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260506T173138867Z-RESOURCE-RAW-ADDRESS-LOWERING-EXCEED-B64EB6D1: Resource raw address lowering exceeds split limit after deep-prefix alias fix

## 概要

After rebasing onto origin/main 4bc486af, node nodesrc/run_source_policy_regressions.js --warn-only reports nodesrc/test_resource_checker_responsibility.js failure: lower_raw_address.rs has 657 lines while the responsibility split limit is 620. The deep-prefix alias fix re-concentrated raw address lowering responsibilities in a module that was previously split.

## 対象

- `nepl-core/src/resource/lower_raw_address.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `origin/main` `4bc486af` 取り込み後、`node nodesrc/run_source_policy_regressions.js --warn-only` は downstream を継続しつつ `nodesrc/test_resource_checker_responsibility.js` の warning を 1 件報告した。
- 失敗内容は `lower_raw_address.rs has 657 lines; responsibility split limit is 620`。
- 同 commit は deep-prefix raw aliasing の性能問題を修正しているが、`lower_raw_address.rs` に raw address alias / return / source lowering の責務が再集約され、既存の責務分割 guard を超過した。
- その後 `origin/main` `376a3bc1` まで取り込んだ再検証でも同じ warning が残っている。

## 問題

After rebasing onto origin/main 4bc486af, node nodesrc/run_source_policy_regressions.js --warn-only reports nodesrc/test_resource_checker_responsibility.js failure: lower_raw_address.rs has 657 lines while the responsibility split limit is 620. The deep-prefix alias fix re-concentrated raw address lowering responsibilities in a module that was previously split.

## 影響

Resource IR raw address lowering is a memory-safety-critical boundary. Letting it grow past the responsibility guard makes future raw pointer/provenance changes harder to audit and weakens the static-check design policy that type and memory safety must remain reviewable.

## 修正方針

Split lower_raw_address.rs by semantic responsibility instead of raising the limit: keep orchestration in lower_raw_address.rs and move deep-prefix alias propagation or raw-address return/source classification into focused modules with their own source-policy guards.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js and node nodesrc/run_source_policy_regressions.js --warn-only with warning 0.
