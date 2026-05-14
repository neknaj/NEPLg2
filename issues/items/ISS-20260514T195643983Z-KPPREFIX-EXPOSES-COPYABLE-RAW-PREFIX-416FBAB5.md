---
id: ISS-20260514T195643983Z-KPPREFIX-EXPOSES-COPYABLE-RAW-PREFIX-416FBAB5
title: "kpprefix exposes copyable raw prefix storage owner"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-15
target: "stdlib/kp/kpprefix.nepl, nodesrc/test_stdlib_kpprefix_owner_boundary.js"
---

# ISS-20260514T195643983Z-KPPREFIX-EXPOSES-COPYABLE-RAW-PREFIX-416FBAB5: kpprefix exposes copyable raw prefix storage owner

## 概要

PrefixI32 stores a raw i32 pointer and length as a public Copy/Clone handle, while prefix_build_i32 and prefix_range_sum_i32 expose raw storage addresses as public APIs. This leaks raw storage identity through a safe KP helper after Stage 6 moved Vec storage ownership to typed owner boundaries.

## 対象

- `stdlib/kp/kpprefix.nepl, nodesrc/test_stdlib_kpprefix_owner_boundary.js`

## 根拠

- `stdlib/kp/kpprefix.nepl` は `core/mem` / `core/mem/internal` / `core/mem/allocator` / `core/mem/raw` を import し、`alloc_raw` / `dealloc_raw` / `load_i32` / `store_i32` を直接使っていた。
- `PrefixI32` は `ptr <i32>` と `len <i32>` だけを持つ handle で、`Copy` / `Clone` を実装していた。これは deallocation responsibility を持つ raw storage handle を shallow copy できる形にしていた。
- `prefix_build_i32` と `prefix_range_sum_i32` は public raw address API だったため、ordinary caller が typed `Vec` observer を迂回して raw prefix storage identity に依存できた。

## 問題

PrefixI32 stores a raw i32 pointer and length as a public Copy/Clone handle, while prefix_build_i32 and prefix_range_sum_i32 expose raw storage addresses as public APIs. This leaks raw storage identity through a safe KP helper after Stage 6 moved Vec storage ownership to typed owner boundaries.

## 影響

Ordinary stdlib users can depend on raw prefix storage identities, duplicate a handle that owns deallocation responsibility, or bypass Vec range/storage-state observers. This conflicts with the Stage 6 MemPtr = non-owning pointer and owner token separation policy.

## 修正方針

Remove public raw prefix builder/query APIs, make PrefixI32 own a Vec<i32> prefix buffer, remove Copy/Clone for PrefixI32, and expose Result-returning build/query wrappers that keep allocation and range failures typed.

## 検証

Add a source policy regression that forbids raw memory imports/helpers in kpprefix and run focused kpprefix doctests plus issue checks.

## 対応内容

- `PrefixI32` を `data <Vec<i32>>` の owner handle に変更し、`Copy` / `Clone` 実装を削除した。
- public raw address API の `prefix_build_i32` / `prefix_range_sum_i32` を削除し、公開面を `prefix_build_vec_i32(Vec<i32>) -> Result<PrefixI32, Diag>` と `prefix_sum_i32(&PrefixI32, i32, i32) -> Result<i32, Diag>` に揃えた。
- 構築時は `vec::filled` で初期化済み prefix buffer を確保し、`vec::get` / `vec::replace` だけで累積和を埋める。range query も `vec::get` を通し、範囲外は `Diag` で返す。
- `nodesrc/test_stdlib_kpprefix_owner_boundary.js` を追加し、raw memory import/helper、raw address API、`PrefixI32` の Copy/Clone 再導入を source policy として禁止した。

## 完了検証

- `node nodesrc/test_stdlib_kpprefix_owner_boundary.js`: pass
- `node nodesrc/test_stdlib_memptr_owner_field_policy.js`: pass
- `node nodesrc/test_stdlib_kpsearch_raw_pointer_boundary.js`: pass
- `node nodesrc/tests.js -i stdlib/kp/kpprefix.nepl --no-tree -o tmp/agent1-kpprefix-vec-owner-boundary-module.json -j 1 --dist web/dist --assert-io`: 1/1 pass

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 6
