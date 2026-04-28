---
id: ISS-20260428T094334365Z-KPSEARCH-VEC-WRAPPERS-USE-RAW-AGGREG-BDDC0E20
title: "kpsearch Vec wrappers use raw aggregate scratch cells under strict move checking"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/kp/kpsearch.nepl, tutorials/getting_started/23_competitive_sort_and_search.n.md"
---

# ISS-20260428T094334365Z-KPSEARCH-VEC-WRAPPERS-USE-RAW-AGGREG-BDDC0E20: kpsearch Vec wrappers use raw aggregate scratch cells under strict move checking

## 概要

lower_bound_vec_i32 / upper_bound_vec_i32 / contains_vec_i32 / count_equal_range_vec_i32 store an owning Vec<i32> into a raw scratch cell only to read data and len. With strict raw ownership checking this is diagnosed as D3100 use of moved raw memory place v_mem.

## 対象

- `stdlib/kp/kpsearch.nepl, tutorials/getting_started/23_competitive_sort_and_search.n.md`

## 根拠

- GitHub Actions run `25045198144` の `tutorials-test` で、`tutorials/getting_started/23_competitive_sort_and_search.n.md::doctest#2` が `stdlib/kp/kpsearch.nepl` の `v_mem` を moved raw memory place として扱う `D3100` で失敗した。
- `lower_bound_vec_i32` / `upper_bound_vec_i32` / `contains_vec_i32` / `count_equal_range_vec_i32` は、`Vec<i32>` の `data` / `len` を読むためだけに所有値全体を raw scratch cell へ `store<Vec<i32>>` していた。

## 問題

lower_bound_vec_i32 / upper_bound_vec_i32 / contains_vec_i32 / count_equal_range_vec_i32 store an owning Vec<i32> into a raw scratch cell only to read data and len. With strict raw ownership checking this is diagnosed as D3100 use of moved raw memory place v_mem.

## 影響

The competitive tutorial doctest fails in GitHub Actions and the wrapper pattern encourages weakening D3100 instead of using borrowed field projection for Copy fields.

## 修正方針

Read Vec data and len through borrowed/ref accessors, call the raw-array search helpers, then release the consumed Vec owner explicitly. Keep the raw memory checker strict.

## 検証

Run the kpsearch focused doctests and the tutorial sort/search doctest.

## 2026-04-28 修正結果

`Vec<i32>` wrapper 4 関数を、raw scratch cell ではなく `data_ptr_ref<i32> &v` と `len_ref<i32> &v` で Copy field だけを借用読みする形へ変更した。raw-array helper を呼び出した後は、関数が受け取った `Vec<i32>` owner を `free<i32> v` で明示的に解放する。

これにより、compiler の D3100 raw memory ownership check を緩めず、wrapper 側の不適切な raw aggregate detour を除去した。

検証:

- `node nodesrc/tests.js -i stdlib/kp/kpsearch.nepl -i tutorials/getting_started/23_competitive_sort_and_search.n.md --no-tree -o tmp/kpsearch-vec-wrapper-borrowed-fields.json -j 1`: total=5, passed=5
