---
id: ISS-20260429T115652973Z-STDIO-READ-HELPERS-TRIGGER-RAWMEMORY-52A45658
title: "stdio read helpers trigger RawMemoryLoadCell ownership violations"
area: stdlib
status: fixed
resolved: true
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

- `stdlib/std/stdio.nepl` の doctest は、`std_load_i32_at__MemPtr_T_u8_i32_i32__Result_T_E_i32_str__pure` と `read_line__unit__str__imp` で `RawMemoryLoadCell ... found Uninit` を報告していた。
- `std_load_i32_at` は任意 `MemPtr<u8>` から `region_ptr_at` で `MemPtr<i32>` を作り、その場で読む汎用 helper だったため、caller 側で初期化済みである事実を ResourceIR が関数境界越しに証明できなかった。
- `read_line` は `std_alloc add 4 cap` で string layout を手作りし、WASI `fd_read` で書かれたはずの byte を raw `load_u8` で読む構造だったため、fd_read out buffer の初期化 provenance が静的検査に残らなかった。

## 問題

Running stdlib/std/stdio.nepl doctests after fixing print_i32 still fails in std_load_i32_at__MemPtr_T_u8_i32_i32__Result_T_E_i32_str__pure and read_line__unit__str__imp with resource.raw.ownership_violation RawMemoryLoadCell diagnostics. This is separate from the print_i32 scratch formatter because tests/compiler/functions.n.md and cargo functions now pass.

## 影響

stdlib/std/stdio.nepl cannot be used as a clean doctest regression target, and read_line/read helper ownership state may hide real stdio safety regressions.

## 修正方針

Review std_load_i32_at and read_line raw-memory paths. Do not weaken RawMemoryLoadCell; make the helper boundary preserve initialized cell state or refactor the read buffer/string construction path so Resource IR can prove the loaded cells are initialized.

## 修正内容

- 任意 pointer を読む `std_load_i32_at` / `std_store_i32_at` helper を削除した。
- `stdio_fd_read_into_result` を追加し、WASI `fd_read` の iov/nread scratch を同一関数内で初期化して読み戻す境界へ閉じ込めた。
- `stdio_finish_read_buffer` を追加し、read buffer を exact-size `ByteBuf` へ shrink してから返すようにした。これにより `ByteBuf.ptr` が `ByteBuf.len` byte の所有領域を指す invariant を満たす。
- `stdio_read_all_bytes_result` は上記 helper を使い、汎用 raw load helper と capacity 過大の `ByteBuf` 返却をやめた。
- `stdio_read_line_result` を追加し、`read_line` は手書き string layout / `string_from_addr_unchecked` を使わず、raw bytes -> exact-size `ByteBuf` -> checked UTF-8 `str` の経路へ変更した。
- `read_line` facade は `stdio_read_line_result` に委譲し、互換 API として失敗時だけ空文字列へ丸める。
- `tests/stdlib/stdio_read_all.n.md` の `ByteBuf` 内容確認は raw `load_u8` ではなく `stdio_write_bytes_result` で stdout に流す形へ修正した。
- `nodesrc/test_stdlib_stdio_read_boundary.js` を追加し、汎用 raw i32 helper と read_line の inline raw string layout 復活を禁止した。

## 検証

- `node nodesrc/test_stdlib_stdio_read_boundary.js`: passed
- `node nodesrc/test_stdlib_stdio_print_i32_boundary.js`: passed
- `node nodesrc/tests.js -i stdlib/std/stdio.nepl --no-tree -o tmp/stdio-read-helper-after-2.json -j 1 --dist web/dist`: `total=28`, `passed=28`, `failed=0`
- `node nodesrc/tests.js -i tests/stdlib/stdio_read_all.n.md --no-tree -o tmp/stdio-read-helper-read-all-3.json -j 1 --dist web/dist`: `total=2`, `passed=2`, `failed=0`
- `node nodesrc/tests.js -i tests/compiler/functions.n.md --no-tree -o tmp/stdio-read-helper-functions.json -j 1 --dist web/dist`: `total=24`, `passed=24`, `failed=0`

## 分離した問題

- `tests/stdlib/stdin.n.md` の StreamScanner 経路は `stdlib/std/streamio.nepl` 側の raw header / byte load で別の `RawMemoryLoadCell` を報告するため、`ISS-20260429T121949904Z-STREAMIO-SCANNER-RAW-BYTE-LOADS-FAIL-B5BB4131` として分離した。
