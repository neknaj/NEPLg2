---
id: ISS-20260604T033642641Z-STDLIB-RAW-IO-BOUNDARIES-DRIFT-FROM--6CE98C14
title: "stdlib raw IO boundaries drift from RegionToken-owned buffer contracts"
area: stdlib
status: open
resolved: false
priority: P1
type: architecture
created: 2026-06-04
updated: 2026-06-04
target: "stdlib/std/fs/stat.nepl, stdlib/std/fs/read/fd.nepl, stdlib/std/stdio/write/fd.nepl, stdlib/std/env/cliarg/cstr.nepl, stdlib/alloc/io/bytebuf"
---

# ISS-20260604T033642641Z-STDLIB-RAW-IO-BOUNDARIES-DRIFT-FROM--6CE98C14: stdlib raw IO boundaries drift from RegionToken-owned buffer contracts

## 概要

Current source policy regressions show stdlib raw IO boundaries no longer satisfy the RegionToken and bounded extent contracts: fs_path_filetype no longer matches the owned stat buffer policy, cstr_len_bounded_result no longer proves 0 <= i < max_len before reading, fs_read_fd_bytes does not finish through the ByteBuf ownership-normalizing helper, and stdio_fd_write_from_result moved scratch ownership into the public helper signature. This conflicts with the Zenn policy that platform and raw-memory effects must stay at explicit boundaries and be statically checked.

## 対象

- `stdlib/std/fs/stat.nepl, stdlib/std/fs/read/fd.nepl, stdlib/std/stdio/write/fd.nepl, stdlib/std/env/cliarg/cstr.nepl, stdlib/alloc/io/bytebuf`

## 根拠

- 未記入

## 問題

Current source policy regressions show stdlib raw IO boundaries no longer satisfy the RegionToken and bounded extent contracts: fs_path_filetype no longer matches the owned stat buffer policy, cstr_len_bounded_result no longer proves 0 <= i < max_len before reading, fs_read_fd_bytes does not finish through the ByteBuf ownership-normalizing helper, and stdio_fd_write_from_result moved scratch ownership into the public helper signature. This conflicts with the Zenn policy that platform and raw-memory effects must stay at explicit boundaries and be statically checked.

## 影響

Raw pointer and host IO layout details can leak through stdlib APIs, weakening Resource IR proof boundaries and making future platform backends depend on ad-hoc buffer conventions.

## 修正方針

Re-normalize raw IO helpers so scratch storage is owned by local RegionToken values, cstr scanning uses an explicit bounded Result path, ByteBuf read completion goes through the ownership-normalizing helper, and public wrappers expose typed Result errors rather than raw layout obligations. Add focused doctests now and cfg-test-style regular tests when that mechanism lands.

## 検証

Run node nodesrc/run_source_policy_regressions.js --warn-only and require the fs, cliarg, io_bytebuf, and stdio read/write boundary warnings to disappear; add focused stdlib tests for invalid pointers, missing NUL, realloc failure, and zero-length writes.
