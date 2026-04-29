---
id: ISS-20260429T064519915Z-STDIO-PRINT-I32-TRIGGERS-RAWMEMORYLO-B90E5FA7
title: "stdio print_i32 triggers RawMemoryLoadCell ownership violation in functions integration test"
area: stdlib
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "stdlib/std/stdio.nepl, nepl-core/tests/functions.rs"
---

# ISS-20260429T064519915Z-STDIO-PRINT-I32-TRIGGERS-RAWMEMORYLO-B90E5FA7: stdio print_i32 triggers RawMemoryLoadCell ownership violation in functions integration test

## 概要

cargo test -p nepl-core --test effects --test functions --test overload -- --nocapture currently fails in function_purity_check_impure_calls_pure. The failure is not a call diagnostic construction change; the compiler reports resource.raw.ownership_violation for print_i32__i32__unit__imp with RawMemoryLoadCell on local scratch found MaybeMoved.

## 対象

- `stdlib/std/stdio.nepl, nepl-core/tests/functions.rs`

## 根拠

- `cargo test -p nepl-core --test effects --test functions --test overload -- --nocapture` で `effects` は 21/21 pass した後、`functions::function_purity_check_impure_calls_pure` が失敗した。
- failure diagnostic は `resource.raw.ownership_violation` で、`print_i32__i32__unit__imp` 内の `RawMemoryLoadCell` が `Local("scratch") ... found MaybeMoved` と報告している。
- 同じ実行で `overload` は別途 8/8 pass しており、typecheck call diagnostic の code-first 化とは独立した stdio / raw memory initialization state の問題として扱う。

## 問題

cargo test -p nepl-core --test effects --test functions --test overload -- --nocapture currently fails in function_purity_check_impure_calls_pure. The failure is not a call diagnostic construction change; the compiler reports resource.raw.ownership_violation for print_i32__i32__unit__imp with RawMemoryLoadCell on local scratch found MaybeMoved.

## 影響

Focused call diagnostics verification cannot use the full functions integration target as a clean regression until stdio print_i32 no longer trips RawMemoryLoadCell. This also risks CI noise around WASI stdio examples that should be valid impure calls.

## 修正方針

Review stdlib/std/stdio.nepl print_i32 and its integer-to-string scratch buffer path. Do not weaken RawMemoryLoadCell; either keep scratch storage initialization/move state provable or split the stdio formatting path so Resource IR can see initialized raw cells.

## 検証

cargo test -p nepl-core --test functions function_purity_check_impure_calls_pure -- --nocapture should pass, followed by cargo test -p nepl-core --test effects --test functions --test overload -- --nocapture.
