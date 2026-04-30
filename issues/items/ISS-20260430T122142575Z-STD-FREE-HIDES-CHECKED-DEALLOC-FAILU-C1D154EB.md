---
id: ISS-20260430T122142575Z-STD-FREE-HIDES-CHECKED-DEALLOC-FAILU-C1D154EB
title: "std_free hides checked dealloc failure from owner summaries"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-30
updated: 2026-04-30
target: "stdlib/std/stdio.nepl, nepl-core/tests/kp.rs"
---

# ISS-20260430T122142575Z-STD-FREE-HIDES-CHECKED-DEALLOC-FAILU-C1D154EB: std_free hides checked dealloc failure from owner summaries

## 概要

std_free matches the safe dealloc Result and returns unit on both Ok and Err. For a valid scratch allocation the Err branch is semantically unreachable, but the function summary must conservatively merge Ok consumption with Err non-consumption and therefore cannot prove the raw owner was freed.

## 対象

- `stdlib/std/stdio.nepl, nepl-core/tests/kp.rs`

## 根拠

- `stdio_write_fd_mem_result` は `std_alloc 8` / `std_alloc 4` の `Result::Ok` payload として `iov` / `nwritten` を受け取り、全正常/失敗 cleanup path で `std_free` を呼んでいた。
- 旧 `std_free` は `dealloc ptr size` を `match` し、`Ok` と `Err` の両方で `()` を返していた。
- Resource IR owner summary は callee 内の `Ok` path では owner consumed、`Err` path では owner retained と保守的に merge するため、caller から見ると `std_free` 後も `MaybeFreed` obligation が残る。
- `std_free` は stdio 内部専用であり、呼び出し元は `std_alloc` の `Result::Ok` から得た valid allocation owner と exact size を渡しているため、checked `dealloc` の失敗を握りつぶす設計自体が owner model と矛盾していた。

## 問題

std_free matches the safe dealloc Result and returns unit on both Ok and Err. For a valid scratch allocation the Err branch is semantically unreachable, but the function summary must conservatively merge Ok consumption with Err non-consumption and therefore cannot prove the raw owner was freed.

## 影響

stdio_write_fd_mem_result leaves iov/nwritten as MaybeLeak under Resource IR owner checking, blocking the KP local scanner fixture and any self-host path using stdio scratch buffers.

## 修正方針

Make std_free an internal unchecked free wrapper over dealloc_raw for callers that already hold a valid raw allocation owner, so the Resource IR summary records unconditional owner consumption instead of hiding a checked dealloc Err branch.

## 検証

- `cargo test -p nepl-core --test kp local_scanner_new_logic_debug -- --nocapture`: passed
- `node nodesrc/tests.js -i tests/stdlib/stdio_result_stderr.n.md --no-tree -o tmp/stdio-result-stderr-std-free.json -j 1 --dist web/dist`: total=3, passed=3
