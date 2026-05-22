---
id: ISS-20260522T090905300Z-VEC-PUSH-FAILURE-MUST-RETURN-REJECTE-21E0522B
title: "Vec push failure must return rejected item owner before non-Copy support"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-22
updated: 2026-05-22
target: "stdlib/alloc/collections/vec/types.nepl, stdlib/alloc/collections/vec/mutation/push.nepl, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js, nodesrc/test_stdlib_collection_cleanup_contract.js"
---

# ISS-20260522T090905300Z-VEC-PUSH-FAILURE-MUST-RETURN-REJECTE-21E0522B: Vec push failure must return rejected item owner before non-Copy support

## 概要

VecPushError<T> returns only the consumed Vec owner. Removing the Copy bound from push would let fallible paths consume a non-Copy item without returning it to the caller, so the API surface cannot express owner recovery for non-Copy payloads.

## 対象

- `stdlib/alloc/collections/vec/types.nepl, stdlib/alloc/collections/vec/mutation/push.nepl, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js, nodesrc/test_stdlib_collection_cleanup_contract.js`

## 根拠

- 親 issue [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md) は、fallible update の失敗時に collection owner と item owner を型で返すことを要求している。
- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 6 は、stdlib module allowlist ではなく Resource IR の generic owner/cell proof boundary による所有権証明を完了条件にしている。
- 既存の `Vec.push<T: Copy>` は成功時の slot initialization を private helper へ閉じたが、failure payload は `Vec<T>` だけを返しており、`item` owner の行方を API 型で表現できていなかった。

## 問題

VecPushError<T> returns only the consumed Vec owner. Removing the Copy bound from push would let fallible paths consume a non-Copy item without returning it to the caller, so the API surface cannot express owner recovery for non-Copy payloads.

## 影響

Non-Copy Vec.push cannot be enabled safely. Self-host AST/HIR/diagnostic collections would either keep Copy-only constraints or reintroduce item leaks/drop ambiguity on allocation/grow failure.

## 修正方針

Introduce a typed rejected-owner payload that carries both Vec<T> and the rejected item T, make VecPushError<T> wrap that payload plus StdErrorKind, and update push/error helpers/source policy so owner recovery is visible before relaxing the Copy bound.

## 検証

Run Vec source policy checks and a focused Vec Resource IR/std test after updating the API.

## 2026-05-22 修正

`VecPushRejected<T>` を追加し、`VecPushError<T>` は `rejected: VecPushRejected<T>` と `error: StdErrorKind` を持つ形へ変更した。`VecPushRejected<T>` は `vec: Vec<T>` と `item: T` を同じ owner recovery payload として保持するため、non-Copy `push` へ進むときに失敗時の `item` が API 型から消えない。

`Vec.push<T: Copy>` の全 failure path は、消費した `Vec<T>` と rejected `item` を `VecPushRejected<T>` に包んで返すようにした。成功 path は従来どおり private `vec_push_slot_store_initialized<T>` に raw store と `collection_slot_initialize_empty` marker を閉じ、stdlib allowlist や marker authority の public 化は追加していない。

`vec_push_error_rejected<T>` を追加し、non-Copy owner recovery は `VecPushRejected<T>` をまとめて返す surface にした。一方、既存の `vec_push_error_vec<T: Copy>` は `item` を一緒に返さないため Copy payload 専用の便宜 accessor として維持し、source policy もその違いを監視する。

検証:

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_collection_cleanup_contract.js`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_vec_push_error_owner_does_not_leak_through_result_err -- --test-threads=1 --exact --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_initialized_check_vec_push_free_closes_stdlib_lifecycle -- --test-threads=1 --exact --nocapture`
