---
id: ISS-20260428T224138753Z-STRING-CONSTRUCTORS-REUSE-REGIONTOKE-91ED01B9
title: "String constructors reuse RegionToken storage after RawMemoryLoadCell move"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-29
target: "stdlib/alloc/string.nepl, nepl-core/src/resource"
---

# ISS-20260428T224138753Z-STRING-CONSTRUCTORS-REUSE-REGIONTOKE-91ED01B9: String constructors reuse RegionToken storage after RawMemoryLoadCell move

## 概要

After the external raw root fix, string parameter reads are improved, but owned string construction still fails: concat_result and from_u128_radix read RegionToken-derived output storage after prior pointer extraction, and RawMemoryLoadCell reports out_region as Moved or scratch_raw as MaybeMoved.

## 対象

- `stdlib/alloc/string.nepl, nepl-core/src/resource`

## 根拠

- `trunk build` 後の `node nodesrc/tests.js -i tests\stdlib\neplg2_type_arena.n.md --no-tree -o tmp\vec-header-ref-reads-after-trunk-type-arena.json -j 1` は `total=5, failed=5` で、全 doctest の top issue が `concat_result__str_str__Result_T_E_str_str__pure` の D3100 だった。
- 具体的には `/stdlib/alloc/string.nepl:552` の `let out_base <MemPtr<u8>> get out_region "ptr"` が `RawMemoryLoadCell ... Local("out_region") ... found Moved` になる。
- `stdlib\alloc\collections\vec.nepl` focused doctest でも、string-heavy helper 経由で同じ `concat_result` D3100 が先に出る。
- 同じログには `/stdlib/alloc/string.nepl:2427` の `from_u128_radix` `out_region` Moved と、`/stdlib/alloc/string.nepl:2434` の `scratch_raw` MaybeMoved も含まれる。
- `ISS-20260428T213732897Z-RAWMEMORYLOADCELL-GATE-REJECTS-INITI-6041ACF2` は `str_addr` parameter backing storage の false Uninit を直したが、owned `RegionToken` / scratch allocation の move state はまだ別問題として残っている。

## 問題

After the external raw root fix, string parameter reads are improved, but owned string construction still fails: concat_result and from_u128_radix read RegionToken-derived output storage after prior pointer extraction, and RawMemoryLoadCell reports out_region as Moved or scratch_raw as MaybeMoved.

## 影響

Self-host TypeArena fixture still fails 5/5 at concat_result, and Vec doctests that import std/test/string-heavy helpers still see string construction D3100 before exercising their own code. This keeps string-heavy parser/resolver/diagnostic work from becoming a reliable regression gate.

## 修正方針

Model RegionToken/owned string allocation as a compiler-owned storage token with separate pointer projections, so reading ptr/data views does not move the owning region and scratch element cells retain initialized state across reverse-copy loops. Do not weaken RawMemoryLoadCell for true moved raw cells.

## 検証

After trunk build, run tests/stdlib/neplg2_type_arena.n.md, stdlib/alloc/string.nepl focused doctests, vec focused doctests, node nodesrc/issues.js check, and git diff --check.
