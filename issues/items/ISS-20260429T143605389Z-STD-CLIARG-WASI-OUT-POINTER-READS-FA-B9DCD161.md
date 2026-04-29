---
id: ISS-20260429T143605389Z-STD-CLIARG-WASI-OUT-POINTER-READS-FA-B9DCD161
title: "std cliarg WASI out pointer reads fail RawMemoryLoadCell gate"
area: stdlib
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "stdlib/std/env/cliarg.nepl, nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js, examples/nm.nepl"
---

# ISS-20260429T143605389Z-STD-CLIARG-WASI-OUT-POINTER-READS-FA-B9DCD161: std cliarg WASI out pointer reads fail RawMemoryLoadCell gate

## 概要

examples/nm.nepl fails nm-compile under the Resource IR RawMemoryLoadCell gate. cliarg_count and cliarg_get pass args_sizes_get/args_get out pointers through MemPtr<i32> projections and then read them after the call, so initialized-cell provenance is lost and RawMemoryLoadCell reports Uninit.

## 対象

- `stdlib/std/env/cliarg.nepl, nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js, examples/nm.nepl`

## 根拠

- `cargo run -p nepl-cli -- --target wasi --profile debug --input examples/nm.nepl --output target/ci-nm` が `cliarg_count` / `cliarg_get` の `load_i32 argc_ptr`、`load_i32 buf_ptr`、`load_i32 arg_slot` で `RawMemoryLoadCell ... found Uninit` を報告した。
- `cli_i32_ptr` は `region_ptr_at` で `MemPtr<i32>` を作り、`args_sizes_get` / `args_get` が書いた out pointer を後続で読み戻していたため、Resource IR が同一 scratch cell の初期化を追跡できなかった。
- `cstr_to_str` は C string を手組みの `[len][bytes]` layout へコピーし、`string_from_addr_unchecked` で `str` にしていたため、失敗 path の dealloc と成功 path の owner transfer が Resource IR から見えにくかった。

## 問題

examples/nm.nepl fails nm-compile under the Resource IR RawMemoryLoadCell gate. cliarg_count and cliarg_get pass args_sizes_get/args_get out pointers through MemPtr<i32> projections and then read them after the call, so initialized-cell provenance is lost and RawMemoryLoadCell reports Uninit.

## 影響

NM compile and any WASI CLI program that imports std/env/cliarg cannot pass strict memory-safety checking. This blocks CI after Source policy succeeds and weakens confidence in argv handling for self-host CLI work.

## 修正方針

Move CLI argv out-pointer scratch initialization and readback into cliarg-specific raw-address boundaries. Avoid constructing MemPtr<i32> out pointers across helper boundaries; initialize scratch cells before WASI/LLVM calls and read raw cells in the same boundary without weakening RawMemoryLoadCell.

## 検証

- `node nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i stdlib/std/env/cliarg.nepl --no-tree -o tmp/cliarg-out-pointer-boundary-docs-2.json -j 1 --dist web/dist`: total=5, passed=5
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed

`cargo run -p nepl-cli -- --target wasi --profile debug --input examples/nm.nepl --output target/ci-nm` は cliarg の `RawMemoryLoadCell ... found Uninit` を報告しなくなった。現在の remaining failure は `stdlib/nm/parser.nepl::document_to_json` と `stdlib/alloc/string.nepl::sb_build_result` の owner obligation leak であり、本 issue の cliarg out-pointer 境界とは別問題として扱う。

## 解決

2026-04-29 に `std/env/cliarg` の argv out-pointer 境界を整理した。

- `CliArgSizes` と `cli_args_sizes_result` を追加し、`args_sizes_get` の scratch store / call / raw readback を同一関数内へ閉じ込めた。
- `cli_i32_ptr` を削除し、`MemPtr<i32>` out-pointer projection を作って関数境界越しに読む設計を廃止した。
- `cliarg_get` は argv pointer array と argv byte buffer を事前初期化し、`args_get` 後に同じ raw address boundary で `arg_slot_raw` を読み戻す。
- `cstr_to_str` は手組み string layout をやめ、`alloc/string::string_from_mem_unchecked_result` へ委譲して string owner transfer を alloc/string の境界に集約した。
- `nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js` に、MemPtr<i32> out pointer helper の再導入禁止、raw-address size boundary、argv scratch 初期化、cstr_to_str の alloc/string 委譲を追加した。
