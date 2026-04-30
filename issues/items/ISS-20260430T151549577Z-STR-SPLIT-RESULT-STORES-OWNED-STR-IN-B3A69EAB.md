---
id: ISS-20260430T151549577Z-STR-SPLIT-RESULT-STORES-OWNED-STR-IN-B3A69EAB
title: "str_split_result stores owned str into raw Vec storage without an element cleanup contract"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-30
updated: 2026-04-30
target: "stdlib/alloc/string.nepl, stdlib/std/fs.nepl, stdlib/tests/string.n.md, nodesrc source policies"
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

## 修正内容

- `str_split_result` / `str_split` を削除し、owned substring を `Vec<str>` raw storage へ `store<str>` する API を廃止した。
- `StrSplitStepKind` enum と `StrSplitStep` struct を追加し、`str_split_next` が `Part` / `Done` を `match` で扱える allocation-free scanner として次の byte range を返す設計にした。
- `str_range_eq` と `sb_append_slice_result` を追加し、range を substring 化せず比較・builder 追加できるようにした。
- `fs_normalize_relative` は `str_split_next` で path component range を走査し、`Vec<i32>` の start/end pair stack と `sb_append_slice_result` で正規 path を組み立てるようにした。
- playground runtime の stdlib export と source policy を更新し、owned `Vec<str>` split API と allocation-bearing split range vector の再導入を拒否する回帰検査を追加した。
- `stdlib/tests/string.n.md` の split 例は `str_split_next` / `str_range_eq` を使う形へ更新し、owned split API に戻らないことを固定した。

## 検証

- `node nodesrc/test_stdlib_string_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_string_doc_no_boilerplate.js`: passed
- `node nodesrc/test_stdlib_fs_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/string.nepl --no-tree -o tmp/str-split-next-string-nepl-current.json -j 1`: `10 total / 10 passed`
- `node nodesrc/tests.js -i stdlib/std/fs.nepl --no-tree -o tmp/str-split-next-fs-nepl-after-cleanup.json -j 1`: `7 total / 7 passed`
- `node nodesrc/tests.js -i stdlib/tests/hashmap.n.md -i stdlib/tests/hashset.n.md -i stdlib/tests/hashset_str.n.md --no-tree -o tmp/str-split-next-hash-nmd-current.json -j 1`: `5 total / 5 passed`
- `node nodesrc/tests.js -i stdlib/tests/string.n.md --no-tree -o tmp/str-split-next-string-nmd-current.json -j 1`: `9 total / 7 passed / 2 failed`
  - 追加・更新した split doctest は passed。
  - 残る 2 件は既存の `std/test` assertion 戻り値不一致と `string_from_utf8_mem_result` の raw owner obligation であり、本 issue の `str_split_result` owner violation ではない。
- `node nodesrc/run_source_policy_regressions.js --warn-only`: 新規 string/fs policy は passed。既存の `owner_summary_variant_paths.rs has 637 lines; responsibility split limit is 380` warning は継続。
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed
