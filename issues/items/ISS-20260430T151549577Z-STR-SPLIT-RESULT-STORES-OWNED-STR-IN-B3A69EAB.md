---
id: ISS-20260430T151549577Z-STR-SPLIT-RESULT-STORES-OWNED-STR-IN-B3A69EAB
title: "str_split_result stores owned str into raw Vec storage without an element cleanup contract"
area: stdlib
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-30
updated: 2026-04-30
target: "stdlib/alloc/string.nepl, stdlib/alloc/collections/vec.nepl, stdlib/alloc/collections/hashmap.nepl, stdlib/alloc/collections/hashset.nepl"
---

# ISS-20260430T151549577Z-STR-SPLIT-RESULT-STORES-OWNED-STR-IN-B3A69EAB: str_split_result stores owned str into raw Vec storage without an element cleanup contract

## 概要

After from_f64_result no longer masks collection doctests, HashMap/HashSet compile reaches str_split_result and fails with resource.raw.ownership_violation at store<str> into Vec<str> raw storage. The function materializes owned substrings and writes them into raw Vec storage, then returns the Vec or deallocates the storage on error without an element-level cleanup/ownership contract that Resource IR can prove.

## 対象

- `stdlib/alloc/string.nepl, stdlib/alloc/collections/vec.nepl, stdlib/alloc/collections/hashmap.nepl, stdlib/alloc/collections/hashset.nepl`

## 根拠

- `node nodesrc/tests.js -i stdlib/alloc/collections/hashmap.nepl --no-tree -o tmp/from-f64-result-hashmap.json -j 1` は `from_f64_result` 修正後、3 doctest すべてで `str_split_result__str_str__Result_T_E_Vec_T_str_str__pure` の `resource.raw.ownership_violation` に進む。
- `node nodesrc/tests.js -i stdlib/alloc/collections/hashset.nepl --no-tree -o tmp/from-f64-result-hashset.json -j 1` も 6 doctest すべてで同じ `str_split_result` failure へ進む。
- 診断位置は `stdlib/alloc/string.nepl` の `store<str> add data_raw mul out_len size_of<str> tail` で、owned `str` を raw `Vec<str>` storage に置く境界が Resource IR 上の owner transfer / cleanup として表現されていない。
- 過去の `ISS-20260430T023401649Z-SELFHOST-REQ-FAILS-STRICT-OWNER-GATE-F0FF69D6` は selfhost_req fixture を `str_find` へ逃がして verified になっているため、現在の `str_split_result` API failure を直接追跡する open issue が必要である。

## 問題

After from_f64_result no longer masks collection doctests, HashMap/HashSet compile reaches str_split_result and fails with resource.raw.ownership_violation at store<str> into Vec<str> raw storage. The function materializes owned substrings and writes them into raw Vec storage, then returns the Vec or deallocates the storage on error without an element-level cleanup/ownership contract that Resource IR can prove.

## 影響

HashMap/HashSet doctests and any caller that still depends on str_split_result cannot serve as clean collection regressions under mandatory memory-safety checking. Leaving this under a broad collection-free issue hides the exact string split API boundary that selfhost code may accidentally copy.

## 修正方針

Redesign str_split_result and Vec<str> ownership together. Either move split output to a typed owned string collection with explicit element cleanup and owned-element transfer, or replace public split users with scanner APIs that avoid Vec<str> ownership when only delimiter positions are needed. Do not weaken Resource IR or store owned str payloads into raw memory without a statically visible owner transfer.

## 検証

Run focused string split, HashMap, and HashSet doctests; add a source policy preventing str_split_result from writing owned str directly into raw Vec storage without a typed owner/cleanup boundary; confirm node nodesrc/issues.js check and git diff --check.
