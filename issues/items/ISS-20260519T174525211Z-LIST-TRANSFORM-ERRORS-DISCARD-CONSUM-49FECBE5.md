---
id: ISS-20260519T174525211Z-LIST-TRANSFORM-ERRORS-DISCARD-CONSUM-49FECBE5
title: "List transform errors discard consumed owner"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-19
updated: 2026-05-19
target: stdlib/alloc/collections/list/transform.nepl
---

# ISS-20260519T174525211Z-LIST-TRANSFORM-ERRORS-DISCARD-CONSUM-49FECBE5: List transform errors discard consumed owner

## 概要

List.map/filter consume a List owner but still return bare Diag on allocation or invariant failure, so callers cannot recover the consumed list owner. This keeps the failure contract outside the type system and conflicts with Stage 6 owner-preserving collection updates.

## 対象

- `stdlib/alloc/collections/list/transform.nepl`

## 根拠

- 未記入

## 問題

List.map/filter consume a List owner but still return bare Diag on allocation or invariant failure, so callers cannot recover the consumed list owner. This keeps the failure contract outside the type system and conflicts with Stage 6 owner-preserving collection updates.

## 影響

A fallible owner-consuming collection transform can hide cleanup/retry policy inside the implementation and regress Resource IR reasoning for owner-backed aggregates. The final non-Copy collection design needs all such boundaries to return owner-bearing error payloads.

## 修正方針

Introduce ListTransformError<T> carrying the original List<T> owner and Diag, update map/filter to return Result<..., ListTransformError<T>>, free only partial output storage on failure, and return the input owner to the caller. Add a source policy regression for owner-consuming fallible collection APIs returning bare Diag/StdErrorKind.

## 検証

Run focused list doctests, the collection cleanup contract doctest, and the stdlib collection source policy.

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- [stdlib collection / mem / string static safety design](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md)

## 解決内容

2026-05-19 に修正した。`List.map` / `List.filter` は入力 `List<T>` owner を消費する fallible transform であるにもかかわらず、allocation failure や storage invariant failure で入力 `Vec<T>` storage を内部 `free` して `Diag` だけを返していた。

修正後は `ListTransformError<T>` を追加し、`map<T, U>` は `Result<List<U>, ListTransformError<T>>`、`filter<T>` は `Result<List<T>, ListTransformError<T>>` を返す。失敗時は部分的に作った出力 `Vec` だけを実装内で閉じ、入力 list owner は `ListTransformError<T>.list` として caller に戻す。成功時だけ入力 storage owner を `free` して出力 list owner を返すため、cleanup / retry の責務が API 型に現れる。

回帰検査として、`nodesrc/test_stdlib_list_no_unsafe_unwraps.js` に `ListTransformError` の構造と `map` / `filter` の owner-preserving result を要求する検査を追加した。また `nodesrc/test_stdlib_collection_cleanup_contract.js` は、generic collection owner を値渡しで消費する fallible API が bare `Diag` / `StdErrorKind` を返す形を型シグネチャから検出する。`tests/stdlib/collection_cleanup_contract.n.md` には `list_transform_error_list<NonCopy>` が `type.trait_bound.unsatisfied` になる compile-fail regression を追加した。
