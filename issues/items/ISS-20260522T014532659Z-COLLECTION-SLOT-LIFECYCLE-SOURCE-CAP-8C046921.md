---
id: ISS-20260522T014532659Z-COLLECTION-SLOT-LIFECYCLE-SOURCE-CAP-8C046921
title: "Collection slot lifecycle source capability treats internal collection APIs as public marker surfaces"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-22
updated: 2026-05-22
target: "nepl-core/src/source_capability/proof.rs, nepl-core/src/loader.rs, nepl-core/tests/effects.rs"
source: doc/neplg2/static_check_complexity_reduction_plan.md#stage-6-stdlib--self-host-準備
---

# ISS-20260522T014532659Z-COLLECTION-SLOT-LIFECYCLE-SOURCE-CAP-8C046921: Collection slot lifecycle source capability treats internal collection APIs as public marker surfaces

## 概要

The collection slot lifecycle source capability proof used ordinary public reachability to classify direct marker users as public surfaces. A public collection API that only calls a private marker helper internally was therefore rejected the same way as a public alias or raw pointer wrapper that exposes marker authority.

## 対象

- `nepl-core/src/source_capability/proof.rs, nepl-core/src/loader.rs, nepl-core/tests/effects.rs`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 6 は、collection payload の lifecycle を stdlib module allowlist ではなく source-derived Resource IR proof boundary へ接続することを要求している。
- [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md) は、non-Copy collection payload support を compiler-owned marker と owner-preserving public API に接続することを残件としている。
- [ISS-20260521T043444492Z-COLLECTION-SLOT-LIFECYCLE-INTRINSIC--58368836](./ISS-20260521T043444492Z-COLLECTION-SLOT-LIFECYCLE-INTRINSIC--58368836.md) は public alias / public wrapper から marker authority が漏れる経路を閉じたが、ordinary call reachability まで public surface と見なしたため collection API 内部実装も拒否していた。

## 問題

The collection slot lifecycle source capability proof used ordinary public reachability to classify direct marker users as public surfaces. A public collection API that only calls a private marker helper internally was therefore rejected the same way as a public alias or raw pointer wrapper that exposes marker authority.

## 影響

Non-Copy collection work is forced toward module or function-name allowlists, or public stdlib APIs cannot connect owner-preserving collection operations to compiler-owned Resource IR markers.

## 修正方針

Separate public marker exposure from ordinary implementation calls. Public aliases and transparent forwarding wrappers remain public surfaces; public raw MemPtr APIs that reach marker helpers are rejected through an ordinary call graph; encapsulated collection APIs without raw pointer public signatures may call private helpers and rely on typecheck/Resource IR proof for safety.

## 検証

Add source capability regression coverage for public aliases, transparent raw wrappers, raw-pointer adapters, and encapsulated collection APIs that use private lifecycle helpers internally.

## 対応結果

2026-05-22 に修正済み。

- collection slot lifecycle source capability の公開判定を、ordinary public reachability から public marker exposure graph へ分離した。
- public alias と transparent forwarding wrapper は引き続き public surface として扱い、private helper の marker authority を再公開できないようにした。
- public signature に `MemPtr` を含む raw-pointer API は ordinary call graph で private helper 到達を追跡し、local adapter を挟んでも marker authority の公開として拒否する。
- public signature が owner aggregate だけを受け渡す collection API は、private lifecycle helper の内部呼び出しだけで source capability を失わない。後段の型検査と Resource IR proof が raw store/load/drop/relocate/storage release を検証する。

## 回帰テスト

- `collection_slot_lifecycle_boundary_is_internal_not_public_surface`
- `collection_slot_lifecycle_intrinsic_rejects_public_alias_surface`
- `collection_slot_lifecycle_intrinsic_rejects_public_stdlib_callable_surface`
- `collection_slot_lifecycle_intrinsic_rejects_public_wrapper_reachability`
- `collection_slot_lifecycle_intrinsic_rejects_public_raw_adapter_reachability`

`collection_slot_lifecycle_boundary_is_internal_not_public_surface` は、direct public、public alias、public alias chain、transparent wrapper、transparent wrapper chain、public raw-pointer adapter、owner aggregate public API の accept/reject を同じ source capability boundary で固定している。
