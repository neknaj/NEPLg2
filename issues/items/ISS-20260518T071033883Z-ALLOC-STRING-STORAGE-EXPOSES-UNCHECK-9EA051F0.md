---
id: ISS-20260518T071033883Z-ALLOC-STRING-STORAGE-EXPOSES-UNCHECK-9EA051F0
title: "alloc string modules expose unchecked str raw-address helpers"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "stdlib/alloc/string/{storage,access,scanner}.nepl, tests/stdlib/memory_safety.n.md, nodesrc/test_stdlib_string_storage_boundary.js"
---

# ISS-20260518T071033883Z-ALLOC-STRING-STORAGE-EXPOSES-UNCHECK-9EA051F0: alloc string modules expose unchecked str raw-address helpers

## 概要

alloc/string/storage still declared unchecked helpers such as string_finish_base, string_addr, and string_from_addr_unchecked as public storage APIs. alloc/string/scanner also exposed scanner_string_addr. A caller that explicitly imports those modules could reach raw address observers or a boundary that constructs str from MemPtr/raw address discipline instead of the RegionToken-consuming string_finish ownership boundary.

## 対象

- `stdlib/alloc/string/storage.nepl`
- `stdlib/alloc/string/access.nepl`
- `stdlib/alloc/string/scanner.nepl`
- `tests/stdlib/memory_safety.n.md`
- `nodesrc/test_stdlib_string_storage_boundary.js`

## 根拠

- `string_finish_base` was no longer called by stdlib; all live string construction paths use `string_finish(RegionToken<u8>, i32)`.
- `string_addr` was only needed by `string_data_ptr` inside storage and by byte access/scanner implementations.
- `scanner_string_addr` was public even though scanner callers only need range and byte classification helpers.
- `doc/neplg2/stdlib_collection_mem_string_static_safety_design.md` already required unchecked string helpers to be internal boundaries.

## 問題

alloc/string modules exposed raw address observers and an obsolete MemPtr-based finalizer as public APIs. This leaked implementation authority that should remain inside compiler-owned raw-memory-boundary modules.

## 影響

The public API surface can bypass the intended RegionToken-to-str ownership proof. This conflicts with Stage 6 memory safety policy because ordinary code should not be able to finalize arbitrary raw addresses or MemPtr views as str values.

## 修正方針

Remove the unused MemPtr-based string_finish_base helper, keep raw address conversion private to string_finish, remove unused header pointer projection, make access/scanner str_addr helpers private, and add regression/source policy coverage that direct imports cannot reach those raw address helpers.

## 修正内容

- `string_finish_base` と未使用の `string_region_len_ptr` を削除した。
- `string_addr` と `string_from_addr_unchecked` は `storage.nepl` 内の private helper にし、`str` への所有権移行は `RegionToken<u8>` を消費する `string_finish` に集約した。
- `access.nepl` は public storage helper に依存せず、private `string_access_addr` から `len` / `string_byte_at_unchecked` を実装する。
- `scanner.nepl` の `scanner_string_addr` を private にし、scanner 利用者へ raw `i32` observer を公開しない。
- source policy と `memory_safety.n.md` に直接 import の回帰を追加した。

## 検証

Run the string storage/access/scanner source policies and focused memory_safety doctests.
