---
id: ISS-20260428T222332284Z-VEC-PUSH-FREE-REJECT-INITIALIZED-HEA-736A6DA9
title: "Vec push/free reject initialized headers under RawMemoryLoadCell gate"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-29
target: stdlib/alloc/collections/vec.nepl
---

# ISS-20260428T222332284Z-VEC-PUSH-FREE-REJECT-INITIALIZED-HEA-736A6DA9: Vec push/free reject initialized headers under RawMemoryLoadCell gate

## 概要

RawMemoryLoadCell gate rejects Vec header field reads in push/free for SelfhostTypeId and SelfhostTypeRecord. During self-host type primitive parity verification, the ty.nepl focused doctest failed at vec.nepl:579 and vec.nepl:1745 in free__Vec_T_T__unit__pure_SelfhostTypeId / SelfhostTypeRecord and push__Vec_T_T_T__Result_T_E_Vec_T_T_StdErrorKind__pure_SelfhostTypeRecord with initialized headers reported as Uninit.

## 対象

- `stdlib/alloc/collections/vec.nepl`

## 根拠

- `node nodesrc/tests.js -i stdlib\neplg2\core\ty\ty.nepl -i stdlib\neplg2\core\builtins\prelude.nepl --no-tree -o tmp\selfhost-type-kind-primitives.json -j 1` で、prelude doctest は通る一方、type arena doctest が compile phase で失敗した。
- 失敗内容は `stdlib/alloc/collections/vec.nepl:1745` の `field::get v "data"` が `free__Vec_T_T__unit__pure_SelfhostTypeId` / `free__Vec_T_T__unit__pure_SelfhostTypeRecord` で `RawMemoryLoadCell ... found Uninit` になるものだった。
- 同じ検証で `stdlib/alloc/collections/vec.nepl:579` の `field::get v "data"` も `push__Vec_T_T_T__Result_T_E_Vec_T_T_StdErrorKind__pure_SelfhostTypeRecord` で `RawMemoryLoadCell ... found Uninit` になった。
- `tests\stdlib\neplg2_type_arena.n.md` では同種の `free` / `push` に加え、`get_ref` の element raw load でも D3100 が出ており、`Vec` の header / backing storage provenance が Resource IR へ十分に渡っていない疑いがある。
- 文字列 storage 側の D3100 は `ISS-20260428T213732897Z-RAWMEMORYLOADCELL-GATE-REJECTS-INITI-6041ACF2` で追跡済みだが、`Vec<SelfhostTypeId>` / `Vec<SelfhostTypeRecord>` の `push` / `free` は別の collection 所有権境界として切り分けて追跡する必要がある。

## 問題

RawMemoryLoadCell gate rejects Vec header field reads in push/free for SelfhostTypeId and SelfhostTypeRecord. During self-host type primitive parity verification, the ty.nepl focused doctest failed at vec.nepl:579 and vec.nepl:1745 in free__Vec_T_T__unit__pure_SelfhostTypeId / SelfhostTypeRecord and push__Vec_T_T_T__Result_T_E_Vec_T_T_StdErrorKind__pure_SelfhostTypeRecord with initialized headers reported as Uninit.

## 影響

Blocks self-host TypeArena verification and any code that owns Vec<T> across push/free under the stricter raw memory gate. It hides real self-host type regressions behind collection header ownership errors.

## 修正方針

Make Vec header/data access go through an ownership/provenance-preserving stdlib boundary or compiler-owned raw memory model; do not silence by dropping free/push calls. Clarify how initialized Vec headers are represented to Resource IR and align the implementation with the parent raw-memory-backed API migration.

## 修正内容

- `Vec` の owner-consuming helpers が `len` / `cap` / `data` header を観察するとき、owned aggregate から `field::get` で field move するのをやめ、`field::get_ref &v` から Copy field として読む形に統一した。
- 対象は `push` / `free` だけでなく、同じ根を持つ `len` / `cap` / `data_ptr` / `data_mem_ptr` / `data_len` / `is_empty` / `get` / `replace` / `pop` / `clear` / `map` / `filter` / `partition` / `take_while` / `drop_while` / `count` / `fold` / `reduce` / `find` / `any` / `all` の header read。
- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` に、`Vec` 実装で `field::get <var> "len|cap|data"` を再導入しない source policy regression を追加した。
- `Vec` backing storage の `load<.T>` が `RawMemoryLoadCell ... found Uninit` になる問題は、header field move とは別の element storage provenance 問題として分離する。

## 検証

- `node nodesrc\test_stdlib_vec_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i stdlib\neplg2\core\ty\ty.nepl -i stdlib\neplg2\core\builtins\prelude.nepl --no-tree -o tmp\vec-header-ref-reads-ty-prelude-after-all.json -j 1`: total=2, passed=2
- `node nodesrc/tests.js -i stdlib\alloc\collections\vec.nepl --no-tree -o tmp\vec-header-ref-reads-vec-after-all.json -j 1`: total=39, passed=13, failed=26。`push` / `free` header D3100 は消えた。残件は既知の string storage D3100 と、`get` / `get_ref` / helper doctest の `load<.T>` backing storage provenance D3100。
- `node nodesrc/tests.js -i tests\stdlib\neplg2_type_arena.n.md --no-tree -o tmp\vec-header-ref-reads-type-arena-after-all.json -j 1`: total=5, failed=5。失敗 top issue は既知の `alloc/string.nepl` `concat_result` D3100 で、`Vec push/free` header D3100 は出ていない。
