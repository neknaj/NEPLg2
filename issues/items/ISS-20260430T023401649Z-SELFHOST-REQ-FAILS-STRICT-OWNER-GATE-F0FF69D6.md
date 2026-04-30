---
id: ISS-20260430T023401649Z-SELFHOST-REQ-FAILS-STRICT-OWNER-GATE-F0FF69D6
title: "selfhost_req fails strict owner gate for string, ByteBuf, and Vec ownership"
area: stdlib
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-30
updated: 2026-04-30
target: "nepl-core/tests/selfhost_req.rs, stdlib/alloc/string.nepl, stdlib/alloc/io.nepl, stdlib/alloc/collections/vec.nepl, stdlib/std/fs.nepl"
---

# ISS-20260430T023401649Z-SELFHOST-REQ-FAILS-STRICT-OWNER-GATE-F0FF69D6: selfhost_req fails strict owner gate for string, ByteBuf, and Vec ownership

## 概要

With the Resource owner gate active, selfhost_req exposes real maybe-leak diagnostics in byte manipulation, string split utilities, and file-read-to-string code. Vec/ByteBuf observers often require consuming owners or returning owned buffers without a consistent destruction path, while str_split_result can partially store owned str values into Vec storage and only deallocate the backing storage on later failure.

## 対象

- `nepl-core/tests/selfhost_req.rs, stdlib/alloc/string.nepl, stdlib/alloc/io.nepl, stdlib/alloc/collections/vec.nepl, stdlib/std/fs.nepl`

## 根拠

- 未記入

## 問題

With the Resource owner gate active, selfhost_req exposes real maybe-leak diagnostics in byte manipulation, string split utilities, and file-read-to-string code. Vec/ByteBuf observers often require consuming owners or returning owned buffers without a consistent destruction path, while str_split_result can partially store owned str values into Vec storage and only deallocate the backing storage on later failure.

## 影響

Self-host code cannot rely on these stdlib APIs under mandatory memory-safety checking. Tests either leak owned buffers/strings or must bypass public APIs with field-level workarounds, and full cargo test cannot be used as a clean regression gate while these ownership contracts remain unresolved.

## 修正方針

Define ownership-safe destruction and observer contracts for str, ByteBuf, and Vec-backed string collections. str_split_result must clean up partially initialized owned elements on failure or avoid storing owned str in raw Vec storage. selfhost_req fixtures should use borrowed observers and explicit frees once the APIs are fixed.

## 検証

cargo test -p nepl-core --test selfhost_req -- --nocapture should pass with no Resource(Owner) diagnostics; cargo test -p nepl-core should no longer stop in selfhost_req after Resource IR tests pass.
