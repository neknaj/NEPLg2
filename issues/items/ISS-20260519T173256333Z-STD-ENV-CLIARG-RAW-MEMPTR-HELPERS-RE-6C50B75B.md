---
id: ISS-20260519T173256333Z-STD-ENV-CLIARG-RAW-MEMPTR-HELPERS-RE-6C50B75B
title: "std/env cliarg raw MemPtr helpers remain public"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-19
updated: 2026-05-19
target: "stdlib/std/env/cliarg/raw.nepl; nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js; tests/stdlib/cliarg_raw_boundary.n.md"
---

# ISS-20260519T173256333Z-STD-ENV-CLIARG-RAW-MEMPTR-HELPERS-RE-6C50B75B: std/env cliarg raw MemPtr helpers remain public

## 概要

std/env/cliarg/raw.nepl still exports implementation-only helpers that take arbitrary MemPtr<u8> plus raw sizes or offsets. The root facade only needs cliarg_count_result and cliarg_get_checked, so exposing cli_args_sizes_result, scratch zeroing, byte loads, and LLVM syscall shims leaves ordinary source able to depend on raw argv internals.

## 対象

- `stdlib/std/env/cliarg/raw.nepl; nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js; tests/stdlib/cliarg_raw_boundary.n.md`

## 根拠

- `stdlib/std/env/cliarg/raw.nepl` の `cli_args_sizes_result` / `cli_zero_u8_buffer_result` / `cli_zero_i32_slots_result` / `cli_load_u8_result` が `pub fn` で、raw argv scratch の `MemPtr<u8>` と size/offset を直接受け取れた。
- LLVM fallback 用の `__cli_copy_to_cstr` / `__cli_open_cmdline` / `__cli_read_cmdline` / `args_sizes_get` / `args_get` も `pub fn` のため、module 内 implementation detail が API surface として残っていた。
- root `std/env/cliarg.nepl` が必要とする公開関数は `cliarg_count_result` と `cliarg_get_checked` だけであり、low-level helper の公開は Stage 6 の `MemPtr = non-owning pointer view` 分離と合っていなかった。

## 問題

std/env/cliarg/raw.nepl still exports implementation-only helpers that take arbitrary MemPtr<u8> plus raw sizes or offsets. The root facade only needs cliarg_count_result and cliarg_get_checked, so exposing cli_args_sizes_result, scratch zeroing, byte loads, and LLVM syscall shims leaves ordinary source able to depend on raw argv internals.

## 影響

The Resource IR can reject many invalid raw spans, but the stdlib API surface still invites direct raw MemPtr use and makes later static-check simplification harder. This weakens the intended separation where MemPtr is a non-owning view internal to a proven raw boundary.

## 修正方針

Keep the root-facing checked helpers public, make implementation-only raw MemPtr and LLVM shim helpers private inside std/env/cliarg/raw, and add source-policy plus compile_fail coverage so these names are not accidentally re-exported.

## 解決内容

- `CliArgSizes` と `cli_args_sizes_result` / `cli_zero_u8_buffer_result` / `cli_zero_i32_slots_result` / `cli_load_u8_result` を private にし、raw argv scratch の pointer/size/offset helper を module 内へ閉じた。
- LLVM fallback の C string / cmdline / `args_sizes_get` / `args_get` helper も private にし、host syscall shim を ordinary source の import surface から外した。
- `nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js` に、これらの helper が `pub` に戻らない source policy を追加した。
- `tests/stdlib/cliarg_raw_boundary.n.md` を追加し、`std/env/cliarg/raw` を direct import しても `cli_args_sizes_result` と `cli_load_u8_result` が見えないことを compile_fail で固定した。

## 対応 stage

- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6: raw argv boundary は `RegionToken` owner と private raw `MemPtr` view へ閉じ、公開 API は checked helper だけに限定する。

## 検証

- `node nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i tests/stdlib/cliarg_raw_boundary.n.md --no-tree -o tmp/agent1-cliarg-raw-boundary-public-helpers.json -j 1 --dist web/dist --assert-io`: 2/2 passed
- `node nodesrc/tests.js -i stdlib/std/env/cliarg.nepl -i stdlib/std/env/cliarg/raw.nepl -i stdlib/tests/cliarg.n.md --no-tree -o tmp/agent1-cliarg-raw-boundary-modules.json -j 1 --dist web/dist --assert-io`: 10/10 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_cliarg_get_accepts_region_token_return_summary -- --exact --nocapture`: passed
