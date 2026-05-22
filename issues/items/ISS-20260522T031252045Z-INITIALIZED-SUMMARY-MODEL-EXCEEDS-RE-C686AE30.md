---
id: ISS-20260522T031252045Z-INITIALIZED-SUMMARY-MODEL-EXCEEDS-RE-C686AE30
title: "initialized summary model exceeds responsibility limit after projection growth"
area: core
status: open
resolved: false
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

## 検証

Run node nodesrc/test_resource_checker_responsibility.js after the split, plus cargo check -p nepl-core and initialized summary focused tests.
