---
id: ISS-20260428T223953830Z-VEC-ELEMENT-LOADS-LOSE-BACKING-STORA-E811458B
title: "Vec element loads lose backing storage initialization under RawMemoryLoadCell gate"
area: stdlib
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-29
target: "stdlib/alloc/collections/vec.nepl, nepl-core/src/resource"
---

# ISS-20260428T223953830Z-VEC-ELEMENT-LOADS-LOSE-BACKING-STORA-E811458B: Vec element loads lose backing storage initialization under RawMemoryLoadCell gate

## 概要

After Vec header reads were moved to field::get_ref, stdlib/alloc/collections/vec.nepl still fails under RawMemoryLoadCell: get(Vec<T>, i32) and get_ref(&Vec<T>, i32) load elements with load<T> from v_data + idx * size_of<T>, but Resource IR reports the backing cell as Uninit even after values were written by push or filled.

## 対象

- `stdlib/alloc/collections/vec.nepl, nepl-core/src/resource`

## 根拠

- `trunk build` 後に `node nodesrc/tests.js -i stdlib\alloc\collections\vec.nepl --no-tree -o tmp\vec-header-ref-reads-after-trunk-vec.json -j 1` を実行し、`total=39, passed=29, failed=10` になった。
- `stdlib\alloc\collections\vec.nepl::doctest#2` は `get__Vec_T_T_i32__Option_T_T__pure_i32` の `load<.T>` が `/stdlib/alloc/collections/vec.nepl:649` で D3100 になり、place は `Local("v_data") ... StorageOffset(?) ... found Uninit` だった。
- `stdlib\alloc\collections\vec.nepl::doctest#7` / `#9` は `get_ref__ref_Vec_T_T_i32__Option_T_T__pure_i32` の `load<.T>` が `/stdlib/alloc/collections/vec.nepl:672` で同じく `v_data` backing storage の `Uninit` になった。
- `ISS-20260428T222332284Z-VEC-PUSH-FREE-REJECT-INITIALIZED-HEA-736A6DA9` で header read は `field::get_ref &v` に統一済みで、`push` / `free` の header D3100 は消えている。残っているのは element cell の初期化範囲を Resource IR が復元できない問題である。
- `ISS-20260428T214527171Z-RESOURCE-IR-RAWMEMORYLOADCELL-GATE-T-F609F5AB` の外部 raw root 修正後も残っているため、function-external raw root ではなく collection-owned backing storage の initialized element range として扱う必要がある。

## 問題

After Vec header reads were moved to field::get_ref, stdlib/alloc/collections/vec.nepl still fails under RawMemoryLoadCell: get(Vec<T>, i32) and get_ref(&Vec<T>, i32) load elements with load<T> from v_data + idx * size_of<T>, but Resource IR reports the backing cell as Uninit even after values were written by push or filled.

## 影響

Vec doctests remain partially blocked (after trunk build, vec.nepl is 29/39 passed and failures include get/get_ref element loads). Self-host arenas, token streams, and diagnostics cannot rely on Vec read APIs as a regression gate while initialized element ranges are invisible to the checker.

## 修正方針

Represent Vec backing storage as an initialized element range owned by the Vec storage token, or introduce a compiler-owned Storage/InitializedCell model that push/filled/store initialize and get/get_ref consume or copy according to T. Do not silence RawMemoryLoadCell globally; preserve load-before-store diagnostics for compiler-owned raw allocations.

## 検証

After trunk build, run vec focused doctests, a small Vec<i32> push/get/get_ref regression, self-host type arena tests, node nodesrc/issues.js check, and git diff --check.
