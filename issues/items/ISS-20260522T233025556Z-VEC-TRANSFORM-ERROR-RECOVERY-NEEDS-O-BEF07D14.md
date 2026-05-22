---
id: ISS-20260522T233025556Z-VEC-TRANSFORM-ERROR-RECOVERY-NEEDS-O-BEF07D14
title: "Vec transform error recovery needs owner-preserving eliminator before non-Copy transform"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-22
updated: 2026-05-22
target: stdlib/alloc/collections/vec/types.nepl
---

# ISS-20260522T233025556Z-VEC-TRANSFORM-ERROR-RECOVERY-NEEDS-O-BEF07D14: Vec transform error recovery needs owner-preserving eliminator before non-Copy transform

## 概要

`VecTransformError<T>` は、transform が消費した入力 `Vec<T>` owner と Copy な `StdErrorKind` を保持する。しかし owner-moving accessor は `vec_transform_error_vec<T: Copy>` だけで、将来 `map` / `filter` / `prefix` / `partition` を non-Copy payload へ拡張するとき、入力 owner と診断情報を同じ control-flow で回収する API 型が不足していた。

`push` / `replace_drop_old` / `pop` では、失敗 payload や結果 payload を consuming eliminator callback で分解する方針に揃えている。transform error だけが Copy-only Vec-only accessor に留まると、後続実装で direct field projection や owner だけを返す accessor を再導入しやすい。

## 対象

- `stdlib/alloc/collections/vec/types.nepl`

## 根拠

- 親 issue [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md) は、non-Copy collection payload を stdlib allowlist ではなく owner-preserving API 型と generic Resource IR proof boundary へ接続することを要求している。
- `VecPushRejected<T>` / `VecReplaceRejected<T>` / `VecPop<T>` は、片方の owner だけを取り出す API ではなく callback eliminator で owner を同時に渡す設計へ整理済みである。
- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) は、静的検査対象の状態と分岐を型付き enum / match / proof boundary へ載せ、文字列や個別 allowlist にしない方針を要求している。

## 問題

`VecTransformError<T>` が owner-backed aggregate であるにもかかわらず、Copy payload 用の `vec_transform_error_vec<T: Copy>` しか public recovery API がなかった。error kind は Copy だが、`Vec<T>` owner と一緒に渡さない設計では、non-Copy transform を追加したときに「owner を返すが診断を捨てる」か「field projection で直接分解する」方向へ流れやすい。

## 影響

non-Copy transform support が Copy-only recovery API によって止まるか、owner-backed error payload の direct field projection を誘発する。これは push / replace / pop で整えた owner-preserving API discipline とずれ、Resource IR summary から owner recovery boundary が読みにくくなる。

## 修正方針

`vec/types.nepl` に `vec_transform_error_with<T, R>` を追加する。これは `VecTransformError<T>` を消費し、`Vec<T>` owner と `StdErrorKind` を同じ callback `(Vec<T>, StdErrorKind)->R` へ渡す。

`vec_transform_error_vec<T: Copy>` は Copy payload 用の便宜 accessor として残す。ただし non-Copy transform の失敗時 recovery は、owner と error kind を同時に渡す `vec_transform_error_with` を使う方針にする。

source policy は、この callback eliminator が存在し、`Vec` owner と error kind を同じ callback へ渡すこと、および Vec-only accessor が Copy-only のままであることを監視する。

## 検証

- `node --check nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node --check nodesrc/test_stdlib_collection_cleanup_contract.js`
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_collection_cleanup_contract.js`
- `node nodesrc/issues.js check --dir issues`
- `node nodesrc/issues.js index --dir issues`
- `cargo fmt --check`
- `git diff --check`

## 対応結果

2026-05-22 に fixed。

- `VecTransformError<T>` に `vec_transform_error_with<T, R>` を追加し、`Vec<T>` owner と `StdErrorKind` を同じ callback 境界へ渡す owner-preserving recovery surface を用意した。
- `vec_transform_error_vec<T: Copy>` は error kind を返さない Vec-only convenience accessor なので Copy-only に留め、non-Copy transform の future recovery path と分離した。
- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` は `vec_transform_error_with` の signature と field extraction / callback 呼び出しを監視する。
- `nodesrc/test_stdlib_collection_cleanup_contract.js` は `vec_transform_error_with` を owner-preserving transform error eliminator として分類し、Copy bound なしの owner surface として認める条件を callback signature に限定した。
