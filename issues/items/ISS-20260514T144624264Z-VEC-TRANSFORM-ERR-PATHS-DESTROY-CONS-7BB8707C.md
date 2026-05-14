---
id: ISS-20260514T144624264Z-VEC-TRANSFORM-ERR-PATHS-DESTROY-CONS-7BB8707C
title: "Vec transform Err paths destroy consumed owner instead of returning it"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-14
target: "stdlib/alloc/collections/vec/transform/**, stdlib/alloc/collections/vec/types.nepl, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js"
---

# ISS-20260514T144624264Z-VEC-TRANSFORM-ERR-PATHS-DESTROY-CONS-7BB8707C: Vec transform Err paths destroy consumed owner instead of returning it

## 概要

Vec.map/filter/partition/take_while/drop_while は owner-consuming かつ fallible だが、allocation failure 時に入力 Vec owner を内部で free して StdErrorKind だけ返す。push や sort_merge_ret と異なり、API 型が owner の行方を表していない。

## 対象

- `stdlib/alloc/collections/vec/transform/**, stdlib/alloc/collections/vec/types.nepl, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`

## 根拠

- `map<T,U>` は `with_capacity<U>` 失敗時に `free<T> v` してから `Result<Vec<U>, StdErrorKind>::Err e` を返していた。
- `filter<T>` / `take_while<T>` / `drop_while<T>` も同様に、出力 Vec の確保失敗で入力 `Vec<T>` を内部破棄していた。
- `partition<T>` は 2 本目の出力 Vec 確保失敗時に部分出力 `left0` と入力 `v` の両方を free し、`StdErrorKind` だけを返していた。
- 既に `push` と `sort_merge_ret` は `VecPushError<T>` / `VecSortMergeError<T>` で入力 owner を返す設計になっており、transform family だけが owner-preserving failure contract から外れていた。

## 問題

Vec.map/filter/partition/take_while/drop_while は owner-consuming かつ fallible だが、allocation failure 時に入力 Vec owner を内部で free して StdErrorKind だけ返す。push や sort_merge_ret と異なり、API 型が owner の行方を表していない。

## 影響

caller は失敗時に入力 Vec を回収・再試行・検査できず、破壊的失敗か owner-preserving 失敗かを型から判定できない。Stage 6 の owner discipline と、fallible update は owner を戻すという方針に反する。

## 修正方針

VecTransformError<T> を導入し、transform API の Err payload に入力 Vec owner と StdErrorKind を持たせる。途中で作成した出力 Vec は内部で free し、入力 Vec は Err で返す。

## 検証

node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js; node nodesrc/test_stdlib_vec_borrowed_observers.js; node nodesrc/tests.js -i stdlib/alloc/collections/vec/transform -i stdlib/tests/vec.n.md -i tests/compiler/list_dot_map.n.md --no-tree -o tmp/agent1-vec-transform-owner-error.json -j 1 --dist web/dist --assert-io

## 修正結果

- `VecTransformError<T>` を追加し、`vec_transform_error_kind` / `vec_transform_error_vec` で error kind と入力 `Vec` owner を分けて取り出せるようにした。
- `map<T,U>` の戻り型を `Result<Vec<U>, VecTransformError<T>>` に変更し、出力確保失敗時は入力 `Vec<T>` を `Err` payload に戻すようにした。
- `filter<T>` / `take_while<T>` / `drop_while<T>` の戻り型を `Result<Vec<T>, VecTransformError<T>>` に変更し、出力確保失敗時は入力 owner を返すようにした。
- `partition<T>` の戻り型を `Result<VecPartition<T>, VecTransformError<T>>` に変更した。2 本目の出力確保に失敗した場合は、部分出力 `left0` だけを内部で free し、入力 `v` は `Err` payload に戻す。
- source policy を更新し、transform family が `StdErrorKind` だけの destructive failure contract に戻らないようにした。

## 検証結果

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: pass
- `node nodesrc/test_stdlib_vec_borrowed_observers.js`: pass
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/transform -i stdlib/tests/vec.n.md -i tests/compiler/list_dot_map.n.md --no-tree -o tmp/agent1-vec-transform-owner-error.json -j 1 --dist web/dist --assert-io`: 15/15 pass
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec --no-tree -o tmp/agent1-vec-transform-owner-error-module.json -j 1 --dist web/dist --assert-io`: 41/41 pass
- `node nodesrc/tests.js -i tests/stdlib/vec_collections.n.md -i tests/stdlib/sort.n.md -i tests/stdlib/sort_simple.n.md --no-tree -o tmp/agent1-vec-transform-owner-error-stdlib.json -j 1 --dist web/dist --assert-io`: 27/27 pass
