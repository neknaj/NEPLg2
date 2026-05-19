---
id: ISS-20260519T130927391Z-VEC-SORT-RAW-HELPERS-ARE-DIRECTLY-CA-BE6B177C
title: "Vec sort raw helpers are directly callable from ordinary source"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-19
updated: 2026-05-19
target: "stdlib/alloc/collections/vec/sort/raw.nepl; stdlib/alloc/collections/vec/sort/raw/{access,quick,heap}.nepl; stdlib/alloc/collections/vec/sort/{quick,heap,common,simple,merge}"
---

# ISS-20260519T130927391Z-VEC-SORT-RAW-HELPERS-ARE-DIRECTLY-CA-BE6B177C: Vec sort raw helpers are directly callable from ordinary source

## 概要

alloc/collections/vec/sort/raw publicly re-exports unchecked MemPtr sort helpers. Ordinary source can import this module and call sort_get_unchecked_data, sort_set_unchecked_data, sort_slice_quick, or sort_heap_sift_down_data with any MemPtr and length, bypassing Vec len/storage checks and initialized-cell discipline.

## 対象

- `stdlib/alloc/collections/vec/sort/raw.nepl; stdlib/alloc/collections/vec/sort/raw/{access,quick,heap}.nepl; stdlib/alloc/collections/vec/sort/{quick,heap,common,simple,merge}`

## 根拠

- 修正前に、ordinary source の probe が `#import "alloc/collections/vec/sort/raw" as raw` と `data_mem_ptr<i32> &v` を組み合わせ、`raw::sort_set_unchecked_data<i32>` / `raw::sort_get_unchecked_data<i32>` を直接呼べることを確認した。
- この経路は canonical `alloc/collections/vec/sort` facade を経由しないため、先行の facade split だけでは通常 source からの unchecked raw sort helper 呼び出しを閉じ切れていなかった。

## 問題

alloc/collections/vec/sort/raw publicly re-exports unchecked MemPtr sort helpers. Ordinary source can import this module and call sort_get_unchecked_data, sort_set_unchecked_data, sort_slice_quick, or sort_heap_sift_down_data with any MemPtr and length, bypassing Vec len/storage checks and initialized-cell discipline.

## 影響

A program can mutate or read Vec backing storage through arbitrary raw spans while the Resource IR observes the raw operation inside compiler-owned stdlib source. This keeps the same direct-import bypass that was removed from vec/raw and weakens memory safety for sort-related raw access.

## 修正方針

Remove the public vec/sort/raw facade and keep raw sort load/store/swap/range traversal inside checked Vec sort implementations only. Update source-policy regressions so raw sort helpers cannot be reintroduced as public/direct-import APIs.

## 検証

- `node nodesrc/test_stdlib_vec_sort_module_split.js`: pass
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: pass
- `node nodesrc/test_stdlib_sort_merge_no_unsafe_unwraps.js`: pass
- `node nodesrc/test_stdlib_vec_borrowed_observers.js`: pass
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/sort -o tmp\agent1-vec-sort-raw-boundary.json --no-tree -j 4`: total=5, passed=5
- `node nodesrc/tests.js -i tests/stdlib/sort_simple.n.md -o tmp\agent1-vec-sort-raw-boundary-sort-simple.json --no-tree -j 4`: total=1, passed=1
- `node nodesrc/test_stdlib_documentation_contract.js`: pass
- `node nodesrc/run_source_policy_regressions.js --warn-only`: pass
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass
- `cargo run -p nepl-cli -- --target wasi --profile debug --input tmp\agent1-vec-sort-raw-direct-import.nepl --output tmp\agent1-vec-sort-raw-direct-import.wasm`: expected failure after `alloc/collections/vec/sort/raw` deletion
- `node nodesrc/tests.js -i tests/stdlib/sort.n.md -o tmp\agent1-vec-sort-raw-boundary-sort.json --no-tree -j 4`: known unrelated failure in `tests\stdlib\sort.n.md::doctest#17`; separated as `ISS-20260519T133155075Z-SORT-DOCTEST-CONSTRUCTS-VECSORTMERGE-9A671F56`

## 対応

- `stdlib/alloc/collections/vec/sort/raw.nepl`、`sort/raw/access.nepl`、`sort/raw/quick.nepl`、`sort/raw/heap.nepl` を削除し、direct-importable な unchecked sort helper module を残さない構成にした。
- quick / heap の raw load/store/swap/range traversal は、`sort_quick` / `sort_heap` と同じ implementation file 内の private helper に閉じた。
- insertion / selection / exchange / gap / merge range は、旧 shared access helper を経由せず、それぞれの検査済み実装内で `data_mem_ptr` から得た non-owning view に対して raw operation を行う。
- source policy regression は、`sort/raw` 復活、`sort_get_unchecked*` / `sort_set_unchecked*` / `sort_swap*` / `sort_slice_quick` の shared helper 復活、private helper の `pub fn` 化を拒否する。
