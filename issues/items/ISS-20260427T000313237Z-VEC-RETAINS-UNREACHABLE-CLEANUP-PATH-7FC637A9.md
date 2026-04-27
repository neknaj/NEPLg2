---
id: ISS-20260427T000313237Z-VEC-RETAINS-UNREACHABLE-CLEANUP-PATH-7FC637A9
title: "Vec retains unreachable cleanup paths in owned buffer internals"
area: stdlib
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/collections/vec.nepl, stdlib/alloc/collections/vec/sort.nepl, tests/stdlib/vec_collections.n.md, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js"
---

# ISS-20260427T000313237Z-VEC-RETAINS-UNREACHABLE-CLEANUP-PATH-7FC637A9: Vec retains unreachable cleanup paths in owned buffer internals

## 概要

Vec.free and scratch-buffer sort cleanup still match dealloc_ptr errors to unreachable instead of using explicit owner-invariant cleanup.

## 対象

- `stdlib/alloc/collections/vec.nepl, stdlib/alloc/collections/vec/sort.nepl, tests/stdlib/vec_collections.n.md`

## 根拠

- `Vec` の `data` buffer は `new` / `with_capacity` / helper allocation が確保し、返された `Vec` owner が単独所有する。
- `Vec.free`、`partition` の途中確保失敗 cleanup、merge sort の scratch buffer cleanup は、所有が確立した buffer を解放する内部処理である。
- しかし `Vec.free` と merge sort は `dealloc_ptr` error を `unreachable` / `Failure` として扱い、`partition` も checked cleanup の error 分岐を握りつぶしていたため、通常内部処理が impossible branch に依存していた。

## 問題

Vec.free and scratch-buffer sort cleanup still match dealloc_ptr errors to unreachable instead of using explicit owner-invariant cleanup.

## 影響

Vec is the central self-host container; unreachable cleanup paths make allocator regressions hard to diagnose and keep normal internals dependent on impossible branches.

## 修正方針

Use dealloc_raw for owned Vec/scratch buffers where ownership is established, keep external allocation failures as Result, and add source and behavior regressions.

## 解決内容

- `Vec.free` を `dealloc_raw mem_ptr_addr v_data ...` に変更し、`cap = 0` の null pointer は `dealloc_raw` の no-op で扱うようにした。
- `partition` の右側 allocation failure cleanup を `dealloc_raw` に変更し、所有済み左 buffer の checked cleanup error 分岐を削除した。
- `sort_merge` / `sort_merge_ret` の scratch buffer cleanup を `dealloc_raw` に変更し、allocation failure だけを `Result::Err` として残した。
- capacity 0 / grow 後 `free` / 再確保 regression と、merge sort scratch cleanup regression、Vec 実装に unsafe unwrap / checked deallocation が戻らない source policy guard を追加した。

## 検証

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec.nepl --no-tree -o tmp/vec-owned-cleanup-docs.json -j 1`: 39/39 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/sort.nepl --no-tree -o tmp/vec-sort-owned-cleanup-docs.json -j 1`: 3/3 passed
- `node nodesrc/tests.js -i tests/stdlib/vec_collections.n.md -i stdlib/tests/vec.n.md --no-tree -o tmp/vec-owned-cleanup-focused.json -j 1`: 4/4 passed
- source policy regressions: pass
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-vec-owned-cleanup.json -j 4`: 298/298 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-vec-owned-cleanup.json -j 4`: 418/418 passed
- `node nodesrc/issues.js check`: pass
