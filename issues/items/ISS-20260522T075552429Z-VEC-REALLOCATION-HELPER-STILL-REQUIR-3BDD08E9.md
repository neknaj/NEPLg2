---
id: ISS-20260522T075552429Z-VEC-REALLOCATION-HELPER-STILL-REQUIR-3BDD08E9
title: "Vec reallocation helper still requires Copy despite storage-only ownership"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-22
updated: 2026-05-22
target: stdlib/alloc/collections/vec/mutation/push.nepl
---

# ISS-20260522T075552429Z-VEC-REALLOCATION-HELPER-STILL-REQUIR-3BDD08E9: Vec reallocation helper still requires Copy despite storage-only ownership

## 概要

Vec grow helper and error-region recovery are still bounded by .T: Copy even though they only move RegionToken<T> storage ownership and do not read payload cells. This blocks non-Copy Vec push/grow work behind a false Copy proof.

## 対象

- `stdlib/alloc/collections/vec/mutation/push.nepl`

## 根拠

- `stdlib/alloc/collections/vec/mutation/push.nepl` の `vec_realloc_region_or_keep<T>` は `RegionToken<T>` の再確保と失敗時 owner 回収だけを扱うにもかかわらず `.T: Copy` を要求していた。
- 同じ file の `vec_realloc_region_error_region<T>` は internal grow failure payload から `RegionToken<T>` を取り出す helper だが、`pub` かつ Copy-bound で、storage-only helper と public collection owner recovery surface の責務が混ざっていた。
- `core/mem` の `realloc_region_bytes_keep<T>` / `region_realloc_error_region<T>` は payload slot を読まず、`RegionToken<T>` owner を成功/失敗どちらかに返す API であり、Copy 境界を必要としない。

## 問題

Vec grow helper and error-region recovery are still bounded by .T: Copy even though they only move RegionToken<T> storage ownership and do not read payload cells. This blocks non-Copy Vec push/grow work behind a false Copy proof.

## 影響

Non-Copy collection payload support cannot progress to grow paths without either duplicating raw memory logic or weakening Resource IR proof boundaries.

## 修正方針

Remove the false Copy bounds from Vec reallocation helper and recovery accessor, add source and Resource IR regressions proving Drop payload can use the storage-only helper without monomorphizing Copy invariant paths.

## 検証

Run Vec source policy tests and focused Resource IR regression for Drop payload reallocation helper.

## 2026-05-22 Agent 1 修正

`VecReallocRegionError<T>`、`vec_realloc_region_error_kind<T>`、`vec_realloc_region_or_keep<T>` を `push.nepl` 内の private 実装境界へ閉じ、`vec_realloc_region_or_keep<T>` から `.T: Copy` 境界を外した。

`vec_realloc_region_error_region<T>` は削除した。grow failure の旧 `RegionToken<T>` owner 回収は public accessor として公開せず、`push` の private grow/push 境界内で `VecPushError<T>` へ包み直す。これにより、non-Copy `RegionToken<T>` storage owner を public recovery surface に出さず、storage-only reallocation helper だけを payload Copy から独立させる。

`nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` は、grow helper が private であること、Copy-bound に戻らないこと、raw realloc を再実装しないことを監視する。`nodesrc/test_stdlib_collection_cleanup_contract.js` は、public owner-returning error accessor の Copy-only policy は維持しつつ、private storage-only RegionToken realloc helper を構造で識別する。

`nepl-core/tests/resource_ir.rs` には、`DropPayload` を持つ `RegionToken<T>` の storage-only realloc helper が Copy raw-access invariant を通らずに monomorphize / Resource IR check できる回帰テストを追加した。
