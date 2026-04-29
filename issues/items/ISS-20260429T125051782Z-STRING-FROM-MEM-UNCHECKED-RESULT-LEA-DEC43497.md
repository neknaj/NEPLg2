---
id: ISS-20260429T125051782Z-STRING-FROM-MEM-UNCHECKED-RESULT-LEA-DEC43497
title: "string_from_mem_unchecked_result leaks output region owner under Resource IR"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "stdlib/alloc/string.nepl, nepl-core/tests/resource_ir.rs, tests/stdlib/stdin.n.md"
---

# ISS-20260429T125051782Z-STRING-FROM-MEM-UNCHECKED-RESULT-LEA-DEC43497: string_from_mem_unchecked_result leaks output region owner under Resource IR

## 概要

After origin/main 78f310e, stdin and streamio focused runs report string_from_mem_unchecked_result leaking the allocated output region owner. This appears separately from the existing concat_result owner issue and blocks scanner token/string conversion validation.

## 対象

- `stdlib/alloc/string.nepl`
- `nepl-core/tests/resource_ir.rs`
- `tests/stdlib/stdin.n.md`

## 根拠

- `string_from_mem_unchecked_result` は `string_alloc_region byte_len` の `Result::Ok region` から本文へ `mem_copy` した後、以前は `string_finish_base out_base byte_len` で返していたため、`region.ptr.raw` の owner が戻り値 `str` へ移ることを Resource IR が証明できなかった。
- `ISS-20260429T122447197Z-STRING-CONCAT-RESULT-LEAKS-OUTPUT-RE-3AA183DE` の修正で `str_from_addr_unchecked` の raw alias と `string_finish(RegionToken, len)` 境界が整備され、`string_from_mem_unchecked_result` も `string_finish region byte_len` へ移行した。
- `tests/stdlib/stdin.n.md` focused run は `total=5`, `passed=5`, `failed=0` まで戻った。

## 問題

After origin/main 78f310e, stdin and streamio focused runs report string_from_mem_unchecked_result leaking the allocated output region owner. This appears separately from the existing concat_result owner issue and blocks scanner token/string conversion validation.

## 影響

Any stdlib or self-host path that copies bytes into a new str can fail the memory-safety gate even when the caller's byte access has been made ResourceIR-safe. This blocks stdin read_line, stream scanner tokens, and string-heavy self-host components.

## 修正方針

Review the string constructor ownership contract. Make the allocated region owner move into the returned str on every Ok path, and make every Err path free or avoid allocating that region. Keep UTF-8 checked and unchecked constructors sharing a single owner-safe construction boundary.

## 修正内容

- `string_from_mem_unchecked_result` の Ok path は、確保済み `RegionToken` を保持したまま本文 copy を行い、最後に `string_finish region byte_len` で `str` へ確定する形へ整理済み。
- `string_from_utf8_mem_result` は同 helper に委譲しているため、checked UTF-8 constructor も同じ owner-safe construction boundary を共有する。
- Resource IR owner 回帰として、`string_from_mem_unchecked_result` が output region owner を `Result::Ok str` に移すことを直接検査するテストを追加した。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_string_from_mem_unchecked_result_transfer -- --nocapture`: passed
- `node nodesrc/tests.js -i tests/stdlib/stdin.n.md --no-tree -o tmp/string-from-mem-stdin-before.json -j 1 --dist web/dist`: `total=5`, `passed=5`, `failed=0`
- `node nodesrc/tests.js -i tests/stdlib/streamio.n.md --no-tree -o tmp/string-from-mem-streamio-before.json -j 1 --dist web/dist`: `string_from_mem_unchecked_result` leak ではなく、既存の `ISS-20260429T123427866Z-STREAMIO-WRITER-RAW-BUFFER-LOADS-FAI-77152BD3` の StreamWriter raw buffer load で失敗する。
