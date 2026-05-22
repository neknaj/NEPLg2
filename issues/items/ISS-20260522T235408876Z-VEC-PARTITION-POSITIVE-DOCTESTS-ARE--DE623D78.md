---
id: ISS-20260522T235408876Z-VEC-PARTITION-POSITIVE-DOCTESTS-ARE--DE623D78
title: "Vec.partition positive doctests are too broad for focused owner-recovery regression"
area: stdlib
status: open
resolved: false
priority: P2
type: performance
created: 2026-05-22
updated: 2026-05-22
target: "stdlib/alloc/collections/vec/transform/filter/partition/view.nepl, stdlib/alloc/collections/vec/transform/filter/partition/build.nepl, nodesrc/tests.js"
---

# ISS-20260522T235408876Z-VEC-PARTITION-POSITIVE-DOCTESTS-ARE--DE623D78: Vec.partition positive doctests are too broad for focused owner-recovery regression

## 概要

`vec_partition_with<T, R>` の陽性 doctest は、通常 source が `partition<T: Copy>` で得た `VecPartition<T>` を eliminator へ渡す形で書くのが意味的に正しい。しかし focused run で `partition` 経由の例が local 240s command budget を超えた。

一方、`VecPartition<T>` を user source で直接構築する例は owner-backed aggregate boundary により `type.owner_aggregate.constructor_restricted` で拒否されるべきなので、成功 doctest にしてはいけない。

## 対象

- `stdlib/alloc/collections/vec/transform/filter/partition/view.nepl`
- `stdlib/alloc/collections/vec/transform/filter/partition/build.nepl`
- `nodesrc/tests.js`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) は、owner-backed aggregate の direct constructor を compiler-owned stdlib boundary に限定する方針を明記している。
- [ISS-20260522T234126652Z-VECPARTITION-RESULT-NEEDS-OWNER-PRES-79F55766](./ISS-20260522T234126652Z-VECPARTITION-RESULT-NEEDS-OWNER-PRES-79F55766.md) で `vec_partition_with<T, R>` を追加したが、fast positive doctest はまだ用意できていない。
- doctest の compile time 問題は timeout 延長や unsafe constructor 公開で隠してはいけない。`partition` の monomorphization、Resource IR summary propagation、または fixture shape のどれが支配的かを切り分ける必要がある。

## 問題

`vec_partition_with<T, R>` の positive runtime path を focused doctest として高速に検証できない。現在は source policy test で signature / field extraction / callback boundary を監視し、doctest では direct constructor 禁止を compile_fail で固定している。

## 影響

`VecPartition<T>` の owner recovery API の陽性 runtime regression が、より広い `partition` doctest に結び付いたままになる。将来 non-Copy partition に進む前に、owner-preserving eliminator の実行経路を単独で見られる fixture へ分離したい。

## 修正方針

`partition` の monomorphization、Resource IR summary propagation、doctest fixture shape のどれが compile time を支配しているかを stage timing 付きで調査する。

修正は以下の条件を満たすこと。

- `VecPartition<T>` の direct constructor を user source に公開しない。
- timeout 引き上げや `skip` で隠さない。
- `partition` 経由、または stdlib 内部 boundary を保った principled fixture により `vec_partition_with<T, R>` の positive path を focused verification できるようにする。
- source policy test は残し、callback signature と owner-preserving extraction の退行を引き続き検出する。

## 検証

- `node nodesrc/run_doctest.js -i stdlib/alloc/collections/vec/transform/filter/partition/view.nepl -n <positive-case> --dist web/dist` が default local case budget 内で通る。
- direct user-source `VecPartition<T>` construction は引き続き `type.owner_aggregate.constructor_restricted` で拒否される。
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_collection_cleanup_contract.js`
