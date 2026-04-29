---
id: ISS-20260429T064519915Z-STDIO-PRINT-I32-TRIGGERS-RAWMEMORYLO-B90E5FA7
title: "stdio print_i32 triggers RawMemoryLoadCell ownership violation in functions integration test"
area: stdlib
status: fixed
resolved: true
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

## 対応結果

`stdlib/std/stdio.nepl` の `print_i32` が独自の raw-memory scratch formatter を持っていたことが原因だった。stdlib には既に `alloc/string::from_i32` があり、stdio が整数文字列化の raw buffer 実装を重複して持つ必要はない。

`print_i32` は `alloc/string` を `string` alias で import し、`print string::from_i32 v` へ委譲する形に修正した。これにより stdio は「文字列化された `str` を出力する」責務だけを持ち、`print_i32` 内の `std_alloc` / `store_u8` / `load_u8` / `string_from_addr_unchecked` scratch path を削除した。

再発防止として `nodesrc/test_stdlib_stdio_print_i32_boundary.js` を追加し、`print_i32` が `alloc/string::from_i32` へ委譲し、local raw-memory scratch formatter を再導入しないことを CI source policy に追加した。

## 2026-04-29 検証

- `node nodesrc/test_stdlib_stdio_print_i32_boundary.js`: pass
- `cargo test -p nepl-core --test functions function_purity_check_impure_calls_pure -- --nocapture`: pass, 1 passed
- `cargo test -p nepl-core --test effects --test functions --test overload -- --nocapture`: pass, effects 21 passed / functions 19 passed / overload 8 passed
- `node nodesrc/tests.js -i tests/compiler/functions.n.md --no-tree -o tmp/stdio-print-i32-functions-nmd.json -j 1 --dist web/dist`: total=24, passed=24, failed=0

補足: `node nodesrc/tests.js -i stdlib/std/stdio.nepl --no-tree -o tmp/stdio-print-i32-boundary.json -j 1 --dist web/dist` は total=28, passed=26, failed=2。残り 2 件は `std_load_i32_at` / `read_line` の別 RawMemoryLoadCell 問題であり、`ISS-20260429T115652973Z-STDIO-READ-HELPERS-TRIGGER-RAWMEMORY-52A45658` として分離した。
