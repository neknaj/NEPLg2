---
id: ISS-20260429T115652973Z-STDIO-READ-HELPERS-TRIGGER-RAWMEMORY-52A45658
title: "stdio read helpers trigger RawMemoryLoadCell ownership violations"
area: stdlib
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: stdlib/std/stdio.nepl
---

# ISS-20260429T115652973Z-STDIO-READ-HELPERS-TRIGGER-RAWMEMORY-52A45658: stdio read helpers trigger RawMemoryLoadCell ownership violations

## 概要

Running stdlib/std/stdio.nepl doctests after fixing print_i32 still fails in std_load_i32_at__MemPtr_T_u8_i32_i32__Result_T_E_i32_str__pure and read_line__unit__str__imp with resource.raw.ownership_violation RawMemoryLoadCell diagnostics. This is separate from the print_i32 scratch formatter because tests/compiler/functions.n.md and cargo functions now pass.

## 対象

- `stdlib/std/stdio.nepl`

## 根拠

- 未記入

## 問題

Running stdlib/std/stdio.nepl doctests after fixing print_i32 still fails in std_load_i32_at__MemPtr_T_u8_i32_i32__Result_T_E_i32_str__pure and read_line__unit__str__imp with resource.raw.ownership_violation RawMemoryLoadCell diagnostics. This is separate from the print_i32 scratch formatter because tests/compiler/functions.n.md and cargo functions now pass.

## 影響

stdlib/std/stdio.nepl cannot be used as a clean doctest regression target, and read_line/read helper ownership state may hide real stdio safety regressions.

## 修正方針

Review std_load_i32_at and read_line raw-memory paths. Do not weaken RawMemoryLoadCell; make the helper boundary preserve initialized cell state or refactor the read buffer/string construction path so Resource IR can prove the loaded cells are initialized.

## 検証

node nodesrc/tests.js -i stdlib/std/stdio.nepl --no-tree -o tmp/stdio-doctest-read-raw-cell.json -j 1 --dist web/dist should pass, plus keep tests/compiler/functions.n.md passing.
