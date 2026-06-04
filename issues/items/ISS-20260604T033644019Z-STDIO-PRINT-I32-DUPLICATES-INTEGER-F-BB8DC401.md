---
id: ISS-20260604T033644019Z-STDIO-PRINT-I32-DUPLICATES-INTEGER-F-BB8DC401
title: "stdio print_i32 duplicates integer formatting instead of using shared formatter boundary"
area: stdlib
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-06-04
updated: 2026-06-04
target: "stdlib/std/stdio/print.nepl, stdlib/alloc/string/integer/format.nepl"
---

# ISS-20260604T033644019Z-STDIO-PRINT-I32-DUPLICATES-INTEGER-F-BB8DC401: stdio print_i32 duplicates integer formatting instead of using shared formatter boundary

## 概要

node nodesrc/test_stdlib_stdio_print_i32_boundary.js reports that print_i32 no longer delegates integer formatting to alloc/string/integer/format::from_i32. The function locally performs sign/digit branching. This conflicts with the Zenn zero-cost abstraction and responsibility-splitting guidance: the same formatting algorithm should live behind the shared typed formatter boundary, not be duplicated in stdio.

## 対象

- `stdlib/std/stdio/print.nepl, stdlib/alloc/string/integer/format.nepl`

## 根拠

- `nodesrc/test_stdlib_stdio_print_i32_boundary.js` は、`print_i32` が `alloc/string/integer/format::from_i32` に委譲していないことを検出していた。
- `stdlib/std/stdio/print.nepl` は `print_i32_negative_digits` を持ち、stdio module 内で digit 生成と符号処理を重複実装していた。
- `stdio` は host output effect の facade であり、整数 formatting は `alloc/string/integer/format` の責務である。

## 問題

node nodesrc/test_stdlib_stdio_print_i32_boundary.js reports that print_i32 no longer delegates integer formatting to alloc/string/integer/format::from_i32. The function locally performs sign/digit branching. This conflicts with the Zenn zero-cost abstraction and responsibility-splitting guidance: the same formatting algorithm should live behind the shared typed formatter boundary, not be duplicated in stdio.

## 影響

Integer formatting behavior can diverge between stdio and alloc/string, and stdio becomes harder to audit because host side effects are mixed with formatting logic.

## 修正方針

Move print_i32 back to the shared integer formatting boundary, keep stdio responsible only for output effects, and add regular tests that compare print_i32 output with formatter output for zero, positive, negative, and i32 min-like boundary values.

## 検証

Run node nodesrc/test_stdlib_stdio_print_i32_boundary.js, focused stdio doctests, and integer format doctests.

## 対応

- `stdlib/std/stdio/print.nepl` に `alloc/string/integer/format` を `string_integer` alias で import した。
- `print_i32` は `print string_integer::from_i32 v` に委譲し、stdio 側は stdout 出力だけを担当する形へ戻した。
- `print_i32_negative_digits` と、整数 digit 生成のためだけに使っていた `core/math/i32` / `core/math/bool` / `std/stdio/write/byte` import を削除した。
- doc comment は「確保なし」の旧契約ではなく、shared formatter 境界と allocation fallback の契約を説明する内容に更新した。

## 検証結果

- `node nodesrc/test_stdlib_stdio_print_i32_boundary.js`: passed
- `node nodesrc/tests.js -i stdlib/std/stdio/print.nepl -i tests/stdlib/stdout.n.md --no-tree -o tmp/agent2-stdio-print-i32-tests.json -j 1 --dist web/dist --assert-io`: total=12, passed=12
- `node nodesrc/run_source_policy_regressions.js --warn-only`: `test_stdlib_stdio_print_i32_boundary.js` passed。既存 warning は 14 件から 13 件に減少した。
