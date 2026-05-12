---
id: ISS-20260512T210823136Z-COLLECTION-COMPILER-FIXTURES-FAIL-AF-70CD17C5
title: "collection compiler fixtures fail after stdlib API and layout changes"
area: TEST
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-12
updated: 2026-05-12
target: "tests/compiler/neplg2.n.md, tests/compiler/sizeof.n.md"
---

# ISS-20260512T210823136Z-COLLECTION-COMPILER-FIXTURES-FAIL-AF-70CD17C5: collection compiler fixtures fail after stdlib API and layout changes

## 概要

During diagnostic D4 coverage verification, the affected-suite run still failed tests/compiler/neplg2.n.md::doctest#33 with type.overload.type_args_mismatch and tests/compiler/sizeof.n.md::doctest#7 with return value 1 instead of 0. These failures were already present before adding diag_code metadata.

## 対象

- `tests/compiler/neplg2.n.md, tests/compiler/sizeof.n.md`

## 根拠

- 未記入

## 問題

During diagnostic D4 coverage verification, the affected-suite run still failed tests/compiler/neplg2.n.md::doctest#33 with type.overload.type_args_mismatch and tests/compiler/sizeof.n.md::doctest#7 with return value 1 instead of 0. These failures were already present before adding diag_code metadata.

## 影響

The diagnostic metadata change can be verified without new failures, but compiler fixture coverage is not clean. Collection API/layout tests can mask real static-check regressions if stale expectations remain.

## 修正方針

Audit current List get API and collection struct layout, then update fixtures or compiler layout logic according to the current stdlib contract. Do not weaken overload or size_of checks to make the tests pass.

## 検証

Run the two focused doctests and the full tests/compiler/neplg2.n.md and tests/compiler/sizeof.n.md files.
