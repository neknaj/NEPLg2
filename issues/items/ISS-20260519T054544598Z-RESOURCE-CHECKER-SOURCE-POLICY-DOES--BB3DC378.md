---
id: ISS-20260519T054544598Z-RESOURCE-CHECKER-SOURCE-POLICY-DOES--BB3DC378
title: "Resource checker source policy does not monitor initialized_alias_offset module"
area: CORE
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-19
updated: 2026-05-19
target: "nodesrc/test_resource_checker_responsibility.js, nepl-core/src/resource/initialized_alias_offset.rs"
---

# ISS-20260519T054544598Z-RESOURCE-CHECKER-SOURCE-POLICY-DOES--BB3DC378: Resource checker source policy does not monitor initialized_alias_offset module

## 概要

run_source_policy_regressions --warn-only reports that initialized_alias_offset.rs is a Resource IR module but is not included in the resource responsibility line-limit monitor.

## 対象

- `nodesrc/test_resource_checker_responsibility.js, nepl-core/src/resource/initialized_alias_offset.rs`

## 根拠

- `node nodesrc/run_source_policy_regressions.js --warn-only` が `initialized_alias_offset.rs must be monitored by resource responsibility line limits` を報告した。
- `nepl-core/src/resource/mod.rs` は `initialized_alias_offset` module を宣言しており、`initialized_alias.rs` から `I32OffsetFacts` を利用している。
- 既存の `nodesrc/test_resource_checker_responsibility.js` は Resource IR module の責務再集中を line limit と module list で監視する方針だが、この新 module が監視対象に入っていない。

## 問題

run_source_policy_regressions --warn-only reports that initialized_alias_offset.rs is a Resource IR module but is not included in the resource responsibility line-limit monitor.

## 影響

Resource IR initialized alias offset proof logic can grow outside the responsibility budget, weakening the source policy that keeps static-check implementation errors visible.

## 修正方針

Register initialized_alias_offset.rs in the resource checker responsibility policy with an appropriate line budget and required module checks, then run the resource policy and source policy regression.

## 検証

node nodesrc/test_resource_checker_responsibility.js; node nodesrc/run_source_policy_regressions.js --warn-only
