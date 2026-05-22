---
id: ISS-20260522T031252045Z-INITIALIZED-SUMMARY-MODEL-EXCEEDS-RE-C686AE30
title: "initialized summary model exceeds responsibility limit after projection growth"
area: core
status: fixed
resolved: true
priority: P2
type: maintenance
created: 2026-05-22
updated: 2026-05-22
target: "nepl-core/src/resource/initialized_summary.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260522T031252045Z-INITIALIZED-SUMMARY-MODEL-EXCEEDS-RE-C686AE30: initialized summary model exceeds responsibility limit after projection growth

## 概要

After splitting initialized_raw_memory.rs, the resource responsibility monitor reaches initialized_summary.rs and fails because the summary model has 81 counted lines while its split limit is 80. Recent raw initialization summary projection fields moved several suffixes to SummaryProjection, bringing the model contract back to the edge of its responsibility budget.

## 対象

- `nepl-core/src/resource/initialized_summary.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- 未記入

## 問題

After splitting initialized_raw_memory.rs, the resource responsibility monitor reaches initialized_summary.rs and fails because the summary model has 81 counted lines while its split limit is 80. Recent raw initialization summary projection fields moved several suffixes to SummaryProjection, bringing the model contract back to the edge of its responsibility budget.

## 影響

The initialized summary data contract can keep accumulating raw-cell, variant, and projection-specific fields in one model module, weakening auditability of Resource IR initialization proofs.

## 修正方針

Review initialized_summary.rs and split any helper-shaped or condition/variant-specific model pieces into focused model modules without raising the current limit unless the core data contract truly requires it.

## 対応内容

`initialized_summary.rs` から variant-specific な raw cell initialization summary model を `initialized_summary_variant_model.rs` へ分離した。`initialized_summary.rs` は function summary と return/param cell contract に集中し、variant payload / requirement / condition の model は専用 module で管理する。

これにより、raw initialization summary の data contract が variant-gated replay の model 詳細を再び吸収して肥大化する状態を解消した。

## 検証

Run node nodesrc/test_resource_checker_responsibility.js after the split, plus cargo check -p nepl-core and initialized summary focused tests.

- `cargo check -p nepl-core`: pass
- `cargo test -p nepl-core resource::initialized_summary_variant_build_tests -- --test-threads=1`: pass
- `cargo test -p nepl-core resource::initialized_summary_apply_param_tests -- --test-threads=1`: pass
- `cargo fmt -p nepl-core --check`: pass
- `node nodesrc/issues.js check --dir issues`: pass
- `git diff --check`: pass
- `node nodesrc/test_resource_checker_responsibility.js`: `initialized_summary.rs` blocker は解消。次の既存 blocker として `initialized_variant.rs has 503-504 counted lines; responsibility split limit is 500` を検出したため、`ISS-20260522T032238561Z-INITIALIZED-VARIANT-MODULE-EXCEEDS-R-FE81E4F9` に分離した。
