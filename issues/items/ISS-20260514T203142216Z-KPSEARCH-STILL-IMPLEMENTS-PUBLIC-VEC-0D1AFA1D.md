---
id: ISS-20260514T203142216Z-KPSEARCH-STILL-IMPLEMENTS-PUBLIC-VEC-0D1AFA1D
title: "kpsearch still implements public Vec helpers through raw storage views"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-15
target: "stdlib/kp/kpsearch.nepl, tests/stdlib/kp.n.md, nodesrc/test_stdlib_kpsearch_raw_pointer_boundary.js"
---

# ISS-20260514T203142216Z-KPSEARCH-STILL-IMPLEMENTS-PUBLIC-VEC-0D1AFA1D: kpsearch still implements public Vec helpers through raw storage views

## 概要

kp/kpsearch no longer exposes raw pointer helpers publicly, but the module still imports core/mem/internal/raw and implements the Vec-facing helpers by converting data_mem_ptr into a raw i32 address. This keeps a public KP utility module as a raw-memory boundary even though lower/upper bound and unique compression can be expressed through typed Vec get/replace operations.

## 対象

- `stdlib/kp/kpsearch.nepl, tests/stdlib/kp.n.md, nodesrc/test_stdlib_kpsearch_raw_pointer_boundary.js`

## 根拠

- `kpsearch` は public raw pointer helper を閉じた後も、module root が `core/mem` / `core/mem/internal` / `core/mem/allocator` / `core/mem/raw` を import していた。
- `lower_bound_vec_i32` / `upper_bound_vec_i32` / `contains_vec_i32` / `count_equal_range_vec_i32` / `unique_sorted_vec_i32` は `data_mem_ptr<i32>` を `mem_ptr_addr` へ変換し、private raw helper へ渡していた。
- 二分探索と sorted unique は `Vec.len` / `Vec.get` / `Vec.replace` だけで記述できるため、KP search module 自体を raw-memory boundary として維持する必要がなかった。

## 問題

kp/kpsearch no longer exposes raw pointer helpers publicly, but the module still imports core/mem/internal/raw and implements the Vec-facing helpers by converting data_mem_ptr into a raw i32 address. This keeps a public KP utility module as a raw-memory boundary even though lower/upper bound and unique compression can be expressed through typed Vec get/replace operations.

## 影響

Resource IR and source-capability proof must continue treating an ordinary KP helper module as raw-memory-capable. That increases the trusted raw boundary surface and leaves search correctness tied to raw storage identity instead of compiler-checkable Vec observer/update APIs.

## 修正方針

Remove raw memory imports and raw i32 helper implementations from kpsearch. Make lower/upper/count/contains borrow &Vec<i32> and use Vec.get for read-only binary search. Keep unique_sorted_vec_i32 as an owner-consuming API but implement its compaction with Vec.get/Vec.replace, preserving the returned owner. Update doctests and KP suite call sites to the borrowed query API.

## 検証

Run kpsearch source policy, focused kpsearch doctests, tests/stdlib/kp.n.md, issue metadata check, and diff whitespace check.

## 修正結果

- `kpsearch` から raw memory import と raw `i32` helper を削除した。
- `lower_bound_vec_i32` / `upper_bound_vec_i32` / `contains_vec_i32` / `count_equal_range_vec_i32` は `&Vec<i32>` を受ける borrowed query API に変更した。
- query 実装は `len<i32>` と `get<i32>` だけを使う二分探索になった。入力 owner は消費しない。
- `unique_sorted_vec_i32` は owner-consuming API のまま維持し、内部 compaction は `get<i32>` / `replace<i32>` で行う。
- `tests/stdlib/kp.n.md` と inline doctest は borrowed query 後に caller が `Vec<i32>` owner を明示的に free する形へ更新した。

## 回帰テスト

- `nodesrc/test_stdlib_kpsearch_raw_pointer_boundary.js` を更新し、raw memory import/helper、raw pointer helper、raw-address based implementation の再導入を拒否するようにした。
- 同テストで query API が `&Vec<i32>` borrowed signature になっていることと、unique API が owner-consuming signature を維持していることを検査する。

## 検証結果

- `node nodesrc/test_stdlib_kpsearch_raw_pointer_boundary.js`
- `node nodesrc/tests.js -i stdlib/kp/kpsearch.nepl --no-tree -o tmp/agent1-kpsearch-vec-boundary-module.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i tests/stdlib/kp.n.md --no-tree -o tmp/agent1-kpsearch-vec-boundary-kp-suite.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/issues.js check --dir issues`
