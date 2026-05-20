---
id: ISS-20260520T165159064Z-RAW-CELL-LIFECYCLE-SOURCE-POLICY-ONL-848A217D
title: "Raw cell lifecycle source policy only checks surface strings"
area: tools
status: open
resolved: false
priority: P2
type: test
created: 2026-05-20
updated: 2026-05-20
target: "nodesrc/test_resource_raw_cell_lifecycle_policy.js, nepl-core/tests/resource_ir.rs"
---

# ISS-20260520T165159064Z-RAW-CELL-LIFECYCLE-SOURCE-POLICY-ONL-848A217D: Raw cell lifecycle source policy only checks surface strings

## 概要

The raw lifecycle source policy checks event names, a match string, and a small forbidden-call list. It passes even when semantic Resource IR regressions such as realloc range transfer failures remain.

## 対象

- `nodesrc/test_resource_raw_cell_lifecycle_policy.js, nepl-core/tests/resource_ir.rs`

## 根拠

- 未記入

## 問題

The raw lifecycle source policy checks event names, a match string, and a small forbidden-call list. It passes even when semantic Resource IR regressions such as realloc range transfer failures remain.

## 影響

A policy test can give false confidence that lifecycle proof was centralized while pre/postconditions are still incomplete. This is especially risky for memory-safety work because enum existence is not enough to prove transition correctness.

## 修正方針

Keep the nodesrc test as an architecture smoke test, but add semantic regression requirements around each lifecycle postcondition and broaden mutation-bypass detection so source policy cannot be the only guard.

## 検証

Add/require focused Rust Resource IR regressions for move-out, store reinitialization, fill, bulk copy, realloc, dealloc, and stale range invalidation; make the nodesrc policy document that it is not the semantic authority.
