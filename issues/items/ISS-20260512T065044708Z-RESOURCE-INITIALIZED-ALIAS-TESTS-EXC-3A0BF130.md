---
id: ISS-20260512T065044708Z-RESOURCE-INITIALIZED-ALIAS-TESTS-EXC-3A0BF130
title: "Resource initialized alias tests exceeds split limit"
area: core
status: open
resolved: false
priority: P2
type: test
created: 2026-05-12
updated: 2026-05-12
target: "nepl-core/src/resource/initialized_alias_tests.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260512T065044708Z-RESOURCE-INITIALIZED-ALIAS-TESTS-EXC-3A0BF130: Resource initialized alias tests exceeds split limit

## 概要

Resource checker responsibility test now reaches initialized_alias_tests.rs, which has 139 lines while the test module limit is 120. The tests cover multiple initialized-alias concerns in one file and need the same responsibility split as production modules.

## 対象

- `nepl-core/src/resource/initialized_alias_tests.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- 未記入

## 問題

Resource checker responsibility test now reaches initialized_alias_tests.rs, which has 139 lines while the test module limit is 120. The tests cover multiple initialized-alias concerns in one file and need the same responsibility split as production modules.

## 影響

Oversized tests make Resource IR memory-safety regression coverage harder to audit and can hide unrelated behavior under a broad test module.

## 修正方針

Do not raise the limit. Split initialized_alias_tests.rs into focused test modules by concern, wire them through resource/mod.rs, and keep line-budget policy enforcing the split.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js, cargo check -p nepl-core --tests, node nodesrc/issues.js check, and git diff --check.
