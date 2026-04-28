---
id: ISS-20260428T213732897Z-RAWMEMORYLOADCELL-GATE-REJECTS-INITI-6041ACF2
title: "RawMemoryLoadCell gate rejects initialized string backing storage"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/resource, stdlib/alloc/string.nepl"
---

# ISS-20260428T213732897Z-RAWMEMORYLOADCELL-GATE-REJECTS-INITI-6041ACF2: RawMemoryLoadCell gate rejects initialized string backing storage

## 概要

After RawMemoryLoadCell became a compiler gate, basic string helpers such as len(str), byte_at, and concat_result fail with D3100 because Resource IR treats str_addr-derived backing storage as an uninitialized raw cell.

## 対象

- `nepl-core/src/resource, stdlib/alloc/string.nepl`

## 根拠

- 未記入

## 問題

After RawMemoryLoadCell became a compiler gate, basic string helpers such as len(str), byte_at, and concat_result fail with D3100 because Resource IR treats str_addr-derived backing storage as an uninitialized raw cell.

## 影響

Any stdlib or self-host module that imports alloc/string or uses str_eq/hash32 can fail during compilation. This blocks string-heavy stdlib tests and self-host resolver/parser registry code even when the source program is memory-safe.

## 修正方針

Teach Resource IR that str_addr returns a read-only initialized string backing address, or introduce a dedicated string storage provenance model so Copy loads from string headers/data are accepted without weakening raw allocation load checks.

## 検証

Run alloc/string focused doctests, tests/stdlib/string.n.md, the self-host builtins prelude doctest, issue index/check, and git diff --check after the gate recognizes string storage.
