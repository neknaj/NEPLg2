---
id: ISS-20260522T234126652Z-VECPARTITION-RESULT-NEEDS-OWNER-PRES-79F55766
title: "VecPartition result needs owner-preserving eliminator before non-Copy partition"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-22
updated: 2026-05-22
target: stdlib/alloc/collections/vec/transform/filter/partition/view.nepl
---

# ISS-20260522T234126652Z-VECPARTITION-RESULT-NEEDS-OWNER-PRES-79F55766: VecPartition result needs owner-preserving eliminator before non-Copy partition

## 概要

`VecPartition<T>` は `matched: Vec<T>` と `rest: Vec<T>` の 2 本の owner を持つ owner-backed aggregate である。しかし public recovery surface は、metadata observer、payload-copying observer、`vec_partition_free<T: Copy>` に限られていた。

現行 `partition<T: Copy>` 本体を non-Copy 化するには move-out / destination initialized prefix / rollback / drop traversal の大きな設計が必要なので、今回は本体の Copy 制約を外さない。一方で、将来 `VecPartition<DropPayload>` を返せるようになったとき、caller が direct field projection や Copy-only free に頼らず 2 本の `Vec<T>` owner を同じ control-flow で回収できる API surface は先に必要である。

## 対象

- `stdlib/alloc/collections/vec/transform/filter/partition/view.nepl`

## 根拠

- 親 issue [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md) は、collection owner payload を stdlib allowlist ではなく owner-preserving API 型と generic Resource IR proof boundary へ接続することを要求している。
- `VecPop<T>` は `vec_pop_with<T, R>` により `Vec<T>` と `Option<T>` owner を同じ callback へ渡す。`VecPartition<T>` も同じ owner-backed aggregate recovery discipline に揃える必要がある。
- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) は、静的検査に載る状態と証明境界を型付き API と match 可能な構造へ寄せる方針を要求している。

## 問題

`VecPartition<T>` の `matched` / `rest` owner を同時に取り出す consuming API がない。`vec_partition_free<T: Copy>` は Copy payload の cleanup 便宜 API としてはよいが、non-Copy partition の前提としては足りない。

## 影響

将来 non-Copy partition を追加したとき、結果 owner aggregate を安全に消費できず、field projection や場当たり的 accessor を増やす圧力になる。これは push / replace / pop / transform error で揃えた owner-preserving callback boundary と不整合になる。

## 修正方針

`partition/view.nepl` に `vec_partition_with<T, R>(VecPartition<T>, (Vec<T>, Vec<T>)*>R)` を追加する。`VecPartition<T>` を消費し、`matched` と `rest` の `Vec<T>` owner を同じ callback へ渡す。

`vec_partition_matched_get<T: Copy>` / `vec_partition_rest_get<T: Copy>` / `vec_partition_free<T: Copy>` は Copy payload 用の便宜 API として維持する。non-Copy partition 本体の設計は別途、move-out / output slot initialization / partial cleanup を Resource IR proof に接続してから行う。

source policy は `vec_partition_with` の signature と field extraction / callback 呼び出しを監視し、Copy bound なしの `VecPartition<T>` owner surface がこの callback boundary に限られることを確認する。

`vec_partition_with` の陽性 doctest は、本来は `partition<T: Copy>` で得た `VecPartition<T>` を渡す形が正しい。しかし focused run では `partition` 経由の positive fixture が local 240s command budget を超えた。通常 source から `VecPartition<T>` を直接構築する例は `type.owner_aggregate.constructor_restricted` で拒否されるべきなので、成功 doctest にはしない。この compile-time / fixture decomposition 問題は [ISS-20260522T235408876Z-VEC-PARTITION-POSITIVE-DOCTESTS-ARE--DE623D78](./ISS-20260522T235408876Z-VEC-PARTITION-POSITIVE-DOCTESTS-ARE--DE623D78.md) として分離する。

## 検証

- `node --check nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node --check nodesrc/test_stdlib_collection_cleanup_contract.js`
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_collection_cleanup_contract.js`
- `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec/transform/filter/partition/view.nepl -n 5 --dist web/dist` (`VecPartition` direct constructor が通常 source で拒否されることを固定する compile_fail doctest)
- `node nodesrc/issues.js check --dir issues`
- `node nodesrc/issues.js index --dir issues`
- `cargo fmt --check`
- `git diff --check`

## 対応結果

2026-05-22 に fixed。

- `vec_partition_with<T, R>` を追加し、`VecPartition<T>` から `matched` / `rest` の両 `Vec<T>` owner を同じ callback へ渡せるようにした。
- `vec_partition_free<T: Copy>` は Copy payload 用 cleanup convenience として残し、non-Copy payload では `vec_partition_with<T, R>` 経由で両 owner を回収する方針を明記した。
- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` は `vec_partition_with` の ownership-preserving signature と実装境界を監視する。
- `nodesrc/test_stdlib_collection_cleanup_contract.js` は `vec_partition_with` を `VecPartition` 専用の owner-preserving recovery surface として分類し、Copy bound なしで認める条件を callback signature に限定した。
- `vec_partition_with` の doctest は direct constructor を成功例にせず、通常 source が owner-backed aggregate を合成できないことを `type.owner_aggregate.constructor_restricted` で固定する。

2026-05-23 追記: [ISS-20260522T235408876Z-VEC-PARTITION-POSITIVE-DOCTESTS-ARE--DE623D78](./ISS-20260522T235408876Z-VEC-PARTITION-POSITIVE-DOCTESTS-ARE--DE623D78.md) で、`vec_partition_from_parts<T>` を stdlib 内の safe constructor boundary として追加した。これは direct constructor 公開ではなく、2 本の `Vec<T>` owner を `VecPartition<T>` へ束ねる API 型を明示するためのもの。positive doctest は `vec_partition_from_parts -> vec_partition_with` の経路で通し、direct constructor compile_fail は維持した。
