---
id: ISS-20260430T043731199Z-VEC-READ-ONLY-OBSERVERS-CONSUME-OWNE-AF1B63AF
title: "Vec read-only observers consume owners instead of borrowing"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-30
updated: 2026-04-30
target: "stdlib/alloc/collections/vec.nepl, stdlib/alloc/collections/vec/sort.nepl, stdlib/tests/vec.n.md, tests/stdlib/vec_collections.n.md, nodesrc/test_stdlib_vec_borrowed_observers.js"
source: doc/neplg2/static_check_complexity_reduction_plan.md#stage-6-stdlib-memory-api-の段階移行
---

# ISS-20260430T043731199Z-VEC-READ-ONLY-OBSERVERS-CONSUME-OWNE-AF1B63AF: Vec read-only observers consume owners instead of borrowing

## 概要

Vec.len, cap, is_empty, get, and data pointer/span observers take Vec by value while duplicate *_ref observers remain as the non-consuming surface.

## 対象

- `stdlib/alloc/collections/vec.nepl, stdlib/alloc/collections/vec/sort.nepl, stdlib/tests/vec.n.md, tests/stdlib/vec_collections.n.md, nodesrc/test_stdlib_vec_borrowed_observers.js`

## 根拠

- `Vec.len` / `cap` / `is_empty` / `get` / `data_ptr` / `data_mem_ptr` / `data_len` は読み取り専用なのに `Vec<.T>` を値で受け取っていた。
- そのため caller は観測だけのために owner を移動し、観測後に `free` する正常な Resource IR 契約を表現できなかった。
- 同じ目的の `*_ref` API が並存しており、canonical surface が分裂していた。
- `pop` / `partition` は owner-bearing な `Vec` を `.Pair` / tuple field で返し、Resource IR の owner obligation が tuple projection に残るケースを誘発していた。

## 問題

Vec.len, cap, is_empty, get, and data pointer/span observers take Vec by value while duplicate *_ref observers remain as the non-consuming surface.

## 影響

Read-only Vec observation can accidentally destroy the collection owner and keeps Stage 6 raw-memory ownership contracts dependent on caller convention rather than the type signature.

## 修正方針

Make read-only Vec observers borrow &Vec, remove duplicate *_ref observer APIs, migrate Vec callers/tests to the canonical borrowed observers, and add source-policy coverage that rejects by-value observers and *_ref surfaces.

## 対応結果

- `len` / `cap` / `is_empty` / `get` / `data_ptr` / `data_mem_ptr` / `data_len` を `&Vec<.T>` receiver に変更し、重複していた `*_ref` observer を削除した。
- raw storage から値を読む `get` / `replace` / `count` / `fold` / `reduce` / `find` / `any` / `all` は `.T: Copy` を明示し、borrowed traversal API として揃えた。
- `pop` は `VecPop<.T>`、`partition` は `VecPartition<.T>` を返すようにし、owner-bearing result を名前付き field で Resource IR に渡す設計へ変更した。
- Vec を内部 storage に使う stack / queue / deque / hashmap / hashset / btree / heap / ringbuffer 等の helper と、examples / tutorials / focused tests を新 API に追従した。
- `nodesrc/test_stdlib_vec_borrowed_observers.js` を追加し、by-value observer と旧 `*_ref` surface の再導入を source policy で検出する。

## 検証

- `node nodesrc/tests.js -i stdlib/alloc/collections/vec.nepl --no-tree -o tmp/vec-borrowed-observers-doctests.json -j 1`: `total=37`, `passed=37`
- `node nodesrc/tests.js -i stdlib/tests/vec.n.md --no-tree -o tmp/vec-borrowed-observers-stdlib-tests.json -j 1`: `total=6`, `passed=6`
- `node nodesrc/tests.js -i tests/stdlib/vec_collections.n.md --no-tree -o tmp/vec-borrowed-observers-collections-tests.json -j 1`: `total=2`, `passed=2`
- `node nodesrc/tests.js -i stdlib/std/fs.nepl --no-tree -o tmp/vec-borrowed-observers-fs-doctests.json -j 1`: `total=7`, `passed=7`
- `node nodesrc/tests.js -i examples/bf.nepl --no-tree -o tmp/vec-borrowed-observers-bf-tests.json -j 1`: `total=2`, `passed=2`
- `node nodesrc/tests.js -i tests/compiler/list_dot_map.n.md -i tests/compiler/overload_nested_generic_push.n.md --no-tree -o tmp/vec-borrowed-observers-compiler-small-tests.json -j 1`: `total=6`, `passed=6`
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_vec_borrowed_observers.js`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed
- main merge 後の再検証:
  - `node nodesrc/tests.js -i stdlib/alloc/collections/vec.nepl --no-tree -o tmp/merge-vec-doctests.json -j 1`: `total=37`, `passed=37`
  - `node nodesrc/tests.js -i stdlib/tests/vec.n.md --no-tree -o tmp/merge-vec-stdlib-tests.json -j 1`: `total=6`, `passed=6`
  - `node nodesrc/tests.js -i tests/stdlib/vec_collections.n.md --no-tree -o tmp/merge-vec-collections-tests.json -j 1`: `total=3`, `passed=3`
  - `node nodesrc/tests.js -i stdlib/std/fs.nepl --no-tree -o tmp/merge-vec-fs-doctests.json -j 1`: `total=7`, `passed=7`
  - `node nodesrc/tests.js -i examples/bf.nepl --no-tree -o tmp/merge-vec-bf-tests.json -j 1`: `total=2`, `passed=2`
  - `node nodesrc/tests.js -i tests/compiler/list_dot_map.n.md -i tests/compiler/overload_nested_generic_push.n.md --no-tree -o tmp/merge-vec-compiler-small-tests.json -j 1`: `total=6`, `passed=6`
