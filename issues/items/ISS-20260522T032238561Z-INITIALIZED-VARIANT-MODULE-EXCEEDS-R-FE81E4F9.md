---
id: ISS-20260522T032238561Z-INITIALIZED-VARIANT-MODULE-EXCEEDS-R-FE81E4F9
title: "initialized variant module exceeds responsibility limit after summary model split"
area: core
status: fixed
resolved: true
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

## 対応内容

`initialized_variant.rs` から pending variant byte-range count の source 解決と caller-side place 具体化を `initialized_variant_count.rs` へ分離した。variant pending initialization 本体は match arm replay / pending state 管理へ集中し、count projection の解釈は専用 module で扱う。

この分割により、variant replay の memory-safety proof と byte-range count instantiation の責務を分け、resource checker responsibility gate を上限緩和なしで通過させた。

## 検証

Run node nodesrc/test_resource_checker_responsibility.js after the split, plus cargo check -p nepl-core and variant/initialized summary focused tests.

- `cargo check -p nepl-core`: pass
- `cargo fmt -p nepl-core --check`: pass
- `cargo test -p nepl-core resource::initialized_summary_variant_build_tests -- --test-threads=1`: pass
- `node nodesrc/test_resource_checker_responsibility.js`: pass
- `node nodesrc/issues.js check --dir issues`: pass
- `git diff --check`: pass
