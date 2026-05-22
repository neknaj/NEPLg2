---
id: ISS-20260522T032238561Z-INITIALIZED-VARIANT-MODULE-EXCEEDS-R-FE81E4F9
title: "initialized variant module exceeds responsibility limit after summary model split"
area: core
status: open
resolved: false
priority: P2
type: maintenance
created: 2026-05-22
updated: 2026-05-22
target: "nepl-core/src/resource/initialized_variant.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260522T032238561Z-INITIALIZED-VARIANT-MODULE-EXCEEDS-R-FE81E4F9: initialized variant module exceeds responsibility limit after summary model split

## 概要

After splitting initialized_summary.rs, the resource responsibility monitor reaches initialized_variant.rs and fails because the variant application module has 503-504 counted lines while its split limit is 500. Variant pending initialization, summary replay, condition instantiation, and raw range transfer remain close enough to the boundary that small static-check changes re-trigger the gate.

## 対象

- `nepl-core/src/resource/initialized_variant.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- 未記入

## 問題

After splitting initialized_summary.rs, the resource responsibility monitor reaches initialized_variant.rs and fails because the variant application module has 503-504 counted lines while its split limit is 500. Variant pending initialization, summary replay, condition instantiation, and raw range transfer remain close enough to the boundary that small static-check changes re-trigger the gate.

## 影響

Variant-gated initialized-state replay can keep accumulating responsibilities in one module, making memory-safety summary application harder to audit.

## 修正方針

Review initialized_variant.rs and split pending variant summary replay helpers into focused modules without raising the current limit.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js after the split, plus cargo check -p nepl-core and variant/initialized summary focused tests.
