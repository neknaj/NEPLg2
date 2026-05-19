---
id: ISS-20260514T204735670Z-VEC-SORT-FACADE-RE-EXPORTS-RAW-MEMPT-6646B4EF
title: "Vec sort facade re-exports raw MemPtr sort helpers"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-14
updated: 2026-05-19
target: "stdlib/alloc/collections/vec/sort*.nepl, stdlib/alloc/collections/vec/sort/**, tests/stdlib/sort.n.md, nodesrc/test_stdlib_vec_sort_module_split.js, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js"
---

# ISS-20260514T204735670Z-VEC-SORT-FACADE-RE-EXPORTS-RAW-MEMPT-6646B4EF: Vec sort facade re-exports raw MemPtr sort helpers

## 概要

The canonical alloc/collections/vec/sort facade re-exports raw MemPtr-based unchecked helpers and the sort_i32 raw slice adapter. Ordinary callers can reach raw storage mutation APIs through the safe Vec sort facade instead of an explicit raw implementation boundary.

## 対象

- `stdlib/alloc/collections/vec/sort*.nepl, stdlib/alloc/collections/vec/sort/**, tests/stdlib/sort.n.md, nodesrc/test_stdlib_vec_sort_module_split.js, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`

## 関連 doc

- [静的検査の不必要な複雑化の解消計画 Stage 6](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- [stdlib collection / memory / string 静的安全設計](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md)

## 根拠

- `stdlib/alloc/collections/vec/sort.nepl` が `sort/common`、`sort/quick`、`sort/heap`、`sort/merge` を一括再公開しており、`sort_get_unchecked_data` / `sort_set_unchecked_data` / `sort_swap_data` / `sort_slice_quick` などの raw `MemPtr` helper が canonical sort facade から利用可能だった。
- `sort_i32` は raw data pointer と length を直接受ける adapter で、`Vec` の safe API ではなく raw storage identity を public API として固定していた。
- Stage 6 の方針では、ordinary `Vec` caller は raw storage identity ではなく safe `Vec` API だけを見る必要がある。

## 問題

The canonical alloc/collections/vec/sort facade re-exports raw MemPtr-based unchecked helpers and the sort_i32 raw slice adapter. Ordinary callers can reach raw storage mutation APIs through the safe Vec sort facade instead of an explicit raw implementation boundary.

## 影響

Safe Vec users can depend on raw storage identity and unchecked bounds/provenance discipline, which keeps Stage 6 raw-memory-backed API migration incomplete and pressures Resource IR to keep recognizing public raw sort helper surfaces.

## 修正方針

Split the Vec sort public facade from raw implementation helpers. Keep safe Vec sort APIs and observers in the canonical facade, keep raw MemPtr traversal inside checked implementation files rather than direct-importable helper facades, remove the sort_i32 public adapter, and update focused tests/source policies to reject raw helper re-export.

## 検証

Run Vec sort source policies, focused sort doctests, and issue index checks.

## 対応

- canonical `alloc/collections/vec/sort` facade は safe `Vec` sort API と observer だけを再公開する構成にした。
- raw unchecked access は当時 `alloc/collections/vec/sort/raw/access`、raw quick-sort traversal は `sort/raw/quick`、raw heap helper は `sort/raw/heap` へ分離したが、後続の `ISS-20260519T130927391Z-VEC-SORT-RAW-HELPERS-ARE-DIRECTLY-CA-BE6B177C` で direct-importable raw submodule 自体も削除した。
- `sort_i32` は互換 alias を残さず削除した。raw traversal は direct-importable helper ではなく、quick / heap / simple / merge の検査済み implementation file 内 private boundary に閉じる。
- `sort/common` の `sort_is_sorted` は `data_mem_ptr` / unchecked read ではなく `Vec.get` / `Option` で読む実装に変え、observer は pure safe boundary のまま維持した。
- `sort/merge` root facade は public API だけを再公開し、buffer/range raw traversal は internal import に閉じた。

## 回帰テスト

- `nodesrc/test_stdlib_vec_sort_module_split.js` に canonical facade / raw helper ownership / `sort_i32` 削除 / merge facade 境界の source policy を追加した。
- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` に sort facade が `MemPtr` / raw traversal helper を公開しない regression を追加した。
- `tests/stdlib/sort.n.md` から raw `sort_i32` runtime doctest を削除し、safe facade から `sort_i32` を参照できない compile-fail regression を追加した。

## 2026-05-19 Agent 1 追記

- `sort/raw` explicit import は safe facade re-export ではないが、ordinary source から直接 import できるため raw helper bypass として不十分だった。
- `ISS-20260519T130927391Z-VEC-SORT-RAW-HELPERS-ARE-DIRECTLY-CA-BE6B177C` で `sort/raw` facade と `sort/raw/{access,quick,heap}` を削除し、`sort_get_unchecked*` / `sort_set_unchecked*` / `sort_swap*` / `sort_slice_quick` の shared helper 復活を source policy で拒否するようにした。

## 検証結果

- `node nodesrc/test_stdlib_vec_sort_module_split.js`: pass
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/sort.nepl --no-tree -o tmp/agent1-vec-sort-facade-module.json -j 1 --dist web/dist --assert-io`: 3/3 pass
- `node nodesrc/tests.js -i tests/stdlib/sort.n.md --no-tree -o tmp/agent1-vec-sort-facade-raw-boundary-sort-tests.json -j 1 --dist web/dist --assert-io`: 20/20 pass
