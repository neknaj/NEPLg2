---
id: ISS-20260428T213732897Z-RAWMEMORYLOADCELL-GATE-REJECTS-INITI-6041ACF2
title: "RawMemoryLoadCell gate rejects initialized string backing storage"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-29
target: "nepl-core/src/resource, stdlib/alloc/string.nepl"
---

# ISS-20260428T213732897Z-RAWMEMORYLOADCELL-GATE-REJECTS-INITI-6041ACF2: RawMemoryLoadCell gate rejects initialized string backing storage

## 概要

After RawMemoryLoadCell became a compiler gate, basic string helpers such as len(str), byte_at, and concat_result fail with D3100 because Resource IR treats str_addr-derived backing storage as an uninitialized raw cell.

## 対象

- `nepl-core/src/resource, stdlib/alloc/string.nepl`

## 根拠

- `tutorials/getting_started/01_hello_world.n.md` が `/stdlib/alloc/string.nepl:450` の `load_i32 string_addr s` で D3100 になっていた。
- 根本原因は `str_addr` の戻り値が `str` parameter の backing storage alias として Resource IR に下がらず、`RawMemoryLoadCell` が temporary deref を未初期化 raw cell と見ていたこと。
- 親対応: [ISS-20260428T214527171Z-RESOURCE-IR-RAWMEMORYLOADCELL-GATE-T-F609F5AB](ISS-20260428T214527171Z-RESOURCE-IR-RAWMEMORYLOADCELL-GATE-T-F609F5AB.md)。

## 問題

After RawMemoryLoadCell became a compiler gate, basic string helpers such as len(str), byte_at, and concat_result fail with D3100 because Resource IR treats str_addr-derived backing storage as an uninitialized raw cell.

## 影響

Any stdlib or self-host module that imports alloc/string or uses str_eq/hash32 can fail during compilation. This blocks string-heavy stdlib tests and self-host resolver/parser registry code even when the source program is memory-safe.

## 修正方針

Teach Resource IR that str_addr returns a read-only initialized string backing address, or introduce a dedicated string storage provenance model so Copy loads from string headers/data are accepted without weakening raw allocation load checks.

## 解決

`str_addr` intrinsic / helper return を raw address alias として lower し、関数 parameter 由来の `str` backing storage を external initialized raw root として扱うようにした。stdlib 側の `alloc/string.nepl` は変更せず、compiler 側で `str` の型不変条件を Resource IR に反映した。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_preserves_str_addr_helper_parameter_raw_load -- --nocapture`: pass
- `trunk build`: pass
- `node nodesrc/tests.js -i tutorials/getting_started/01_hello_world.n.md --no-tree -o tmp/raw-load-cell-hello-world.json -j 1`: 1 passed
