---
id: ISS-20260517T183118826Z-RESOURCE-IR-TESTS-STILL-TREAT-NONFAT-13A56E1A
title: "Resource IR tests still treat nonfatal warnings as typecheck failures"
area: tests
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-17
updated: 2026-05-17
target: nepl-core/tests/resource_ir.rs
---

# ISS-20260517T183118826Z-RESOURCE-IR-TESTS-STILL-TREAT-NONFAT-13A56E1A: Resource IR tests still treat nonfatal warnings as typecheck failures

## 概要

The resource_ir.rs typecheck_resource_source helper asserts that checked.diagnostics is empty, so stdlib shadow warnings emitted by the modern diagnostic pipeline make focused Resource IR tests fail before Resource IR checks run.

## 対象

- `nepl-core/tests/resource_ir.rs`

## 根拠

- `cargo test -p nepl-core resource_ir_cell_check_preserves_direct_arithmetic_external_raw_load -- --nocapture` が、Resource IR の本体検査に入る前に `resolve.shadow.outer_definition` warning だけで失敗していた。
- 同じ source は warning を含むが type error ではなく、開発方針上は後続の Resource IR 検査を止めるべきではない。

## 問題

The resource_ir.rs typecheck_resource_source helper asserts that checked.diagnostics is empty, so stdlib shadow warnings emitted by the modern diagnostic pipeline make focused Resource IR tests fail before Resource IR checks run.

## 影響

Compiler static-check regressions cannot be isolated with focused tests when unrelated warning diagnostics are present. This contradicts the current CI/development rule that warnings must be visible but must not stop later compile/test/deploy work.

## 修正方針

Change the Resource IR test helper to reject only diagnostics with Error severity while still showing any diagnostics in the assertion message. This keeps real type errors fatal and lets warnings remain nonfatal evidence.

## 検証

cargo test -p nepl-core resource_ir_cell_check_preserves_direct_arithmetic_external_raw_load -- --nocapture

## 対応内容

- `typecheck_resource_source_with_target` の前提を「diagnostics が空」から「error severity が存在しない」へ変更した。
- warning は assertion message には残るため、非fatalとして観測可能なまま Resource IR の focused test を継続できる。

## 検証結果

- `cargo test -p nepl-core resource_ir_cell_check_preserves_direct_arithmetic_external_raw_load -- --nocapture`: passed
