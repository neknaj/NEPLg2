---
id: ISS-20260604T033642641Z-STDLIB-RAW-IO-BOUNDARIES-DRIFT-FROM--6CE98C14
title: "stdlib raw IO boundaries drift from RegionToken-owned buffer contracts"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-06-04
updated: 2026-06-04
target: "stdlib/std/fs/stat.nepl, stdlib/std/fs/read/fd.nepl, stdlib/std/stdio/write/fd.nepl, stdlib/std/env/cliarg/cstr.nepl, stdlib/alloc/io/bytebuf, nodesrc raw-IO source policies"
---

# ISS-20260604T033642641Z-STDLIB-RAW-IO-BOUNDARIES-DRIFT-FROM--6CE98C14: stdlib raw IO boundaries drift from RegionToken-owned buffer contracts

## 概要

Current source policy regressions reported that stdlib raw IO boundaries no longer satisfy the RegionToken and bounded extent contracts. Inspection showed the stdlib implementation already keeps those contracts: raw/platform effects are private, owner obligations stay on `RegionToken`, public APIs expose typed `Result`, and external bytes are checked before string construction. The failure was caused by stale source policies that expected old helper locations, old local variable names, and old scratch `MemPtr` helper signatures instead of the current borrowed-`RegionToken` boundary.

## 対象

- `stdlib/std/fs/stat.nepl, stdlib/std/fs/read/fd.nepl, stdlib/std/stdio/write/fd.nepl, stdlib/std/env/cliarg/cstr.nepl, stdlib/alloc/io/bytebuf`

## 根拠

- `fs_path_filetype_normalized` owns/deallocs `stat_region`, while public `fs_path_filetype` normalizes before entering that raw stat boundary.
- `fs_read_fd_bytes` owns the growable buffer and iovec/nread scratch through `RegionToken` and finishes via `fs_finish_read_buffer`.
- `cstr_len_bounded_result` proves the bounded loop with `pointer_valid`; `cstr_to_str_bounded_result` uses the measured length for UTF-8 validation before `str` construction.
- `stdio_fd_write_from_result` and `fs_fd_write_from_result` are private helpers that borrow `&RegionToken u8` scratch owners. They no longer accept caller-selected scratch `MemPtr` pairs.

## 問題

The source policies were stale and overfit implementation spelling. They required raw stat logic in `fs_path_filetype` rather than the normalized private helper, required local names such as `cap`, `ok`, `len`, and `err`, and expected scratch `MemPtr` arguments where current code correctly borrows `RegionToken` scratch owners.

## 影響

Raw pointer and host IO layout details can leak through stdlib APIs, weakening Resource IR proof boundaries and making future platform backends depend on ad-hoc buffer conventions.

## 修正方針

Keep stdlib source unchanged. Update the source policies to check the actual ownership relations: normalized private raw boundaries, captured bounded-loop state variables, growable buffer completion through `fs_finish_read_buffer`, and borrowed `RegionToken` scratch helpers for fd read/write.

## 検証

- `node nodesrc/test_stdlib_fs_no_unsafe_unwraps.js`: pass
- `node nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js`: pass
- `node nodesrc/test_stdlib_io_bytebuf_owner_boundary.js`: pass
- `node nodesrc/test_stdlib_stdio_read_boundary.js`: pass
- `node nodesrc/tests.js -i stdlib/std/fs/stat.nepl -i stdlib/std/fs/read/fd.nepl -i stdlib/std/fs/write/fd.nepl -i stdlib/std/env/cliarg/cstr.nepl -i stdlib/alloc/io/bytebuf.nepl --no-tree -o tmp/agent2-raw-io-boundary-focused.json -j 1 --dist web/dist --assert-io`: total=5, passed=5, failed=0
- `node nodesrc/run_source_policy_regressions.js --warn-only`: raw IO warnings disappeared。既存 warning は 11 件から 7 件へ減少
- `node nodesrc/issues.js index --dir issues && node nodesrc/issues.js check --dir issues`: pass
- `git diff --check`: pass
- `trunk build`: pass
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/agent2-raw-io-boundary-playground-editor.json`: 13/13 pass
