---
id: ISS-20260428T222332284Z-VEC-PUSH-FREE-REJECT-INITIALIZED-HEA-736A6DA9
title: "Vec push/free reject initialized headers under RawMemoryLoadCell gate"
area: stdlib
status: open
resolved: false
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

## 検証

Run ty/prelude focused doctests, vec focused doctests, and a regression that allocates Vec<SelfhostTypeId>/Vec<SelfhostTypeRecord>, pushes values, and frees successfully under RawMemoryLoadCell gate.
