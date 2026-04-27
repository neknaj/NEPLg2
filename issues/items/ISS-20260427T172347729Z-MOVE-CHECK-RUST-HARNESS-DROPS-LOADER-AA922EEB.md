---
id: ISS-20260427T172347729Z-MOVE-CHECK-RUST-HARNESS-DROPS-LOADER-AA922EEB
title: "move_check Rust harness drops loader source capabilities"
area: core
status: open
resolved: false
priority: P1
type: test
created: 2026-04-27
updated: 2026-04-27
target: nepl-core/tests/move_check.rs
---

# ISS-20260427T172347729Z-MOVE-CHECK-RUST-HARNESS-DROPS-LOADER-AA922EEB: move_check Rust harness drops loader source capabilities

## 概要

nepl-core/tests/move_check.rs loads stdlib through Loader but calls compile_module without the Loader SourceMap. After core/mem raw memory boundary moved to SourceCapabilities, the harness rejects audited stdlib core/mem raw bodies before move/borrow assertions run.

## 対象

- `nepl-core/tests/move_check.rs`

## 根拠

- 未記入

## 問題

nepl-core/tests/move_check.rs loads stdlib through Loader but calls compile_module without the Loader SourceMap. After core/mem raw memory boundary moved to SourceCapabilities, the harness rejects audited stdlib core/mem raw bodies before move/borrow assertions run.

## 影響

Borrow/lifetime/move regression tests can all fail with TypePureCallsImpureFunction and stop monitoring the memory-safety checks they are meant to cover.

## 修正方針

Pass loaded.source_map to compile_module_with_source_map or a shared source-map aware helper so imported stdlib modules keep their raw memory boundary capabilities.

## 検証

Run cargo test -p nepl-core --test move_check and keep the source capability behavior covered by effects tests.
