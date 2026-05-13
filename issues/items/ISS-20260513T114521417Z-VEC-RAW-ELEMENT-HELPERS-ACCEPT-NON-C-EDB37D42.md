---
id: ISS-20260513T114521417Z-VEC-RAW-ELEMENT-HELPERS-ACCEPT-NON-C-EDB37D42
title: "Vec raw element helpers accept non-Copy payloads"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-13
updated: 2026-05-13
target: stdlib/alloc/collections/vec/raw/element.nepl
---

# ISS-20260513T114521417Z-VEC-RAW-ELEMENT-HELPERS-ACCEPT-NON-C-EDB37D42: Vec raw element helpers accept non-Copy payloads

## 概要

vec_read_at<T> and vec_write_at<T> are public raw Vec element helpers that directly perform typed load/store through MemPtr<T> without requiring T: Copy. Even after public Vec operations are Copy-gated, direct import of vec/raw can still shallow-read or overwrite non-Copy payloads without initialized-cell/drop proof.

## 対象

- `stdlib/alloc/collections/vec/raw/element.nepl`

## 根拠

- `vec/raw.nepl` は `raw/element.nepl` を `pub #import ... as @merge` で再公開している。
- `Vec` root facade も raw module を含む implementation module を public facade として扱うため、`vec_read_at` / `vec_write_at` は単なる private helper ではない。
- 変更前の `vec_read_at<T>` / `vec_write_at<T>` は raw typed `load<T>` / `store<T>` を直接実行する一方で `.T` に制約がなかった。
- `push` / `pop` / `sort` などの public API を Copy-only にしても、raw helper が non-Copy payload を受け入れると同じ shallow read / overwrite 問題を direct import で再導入できる。

## 問題

vec_read_at<T> and vec_write_at<T> are public raw Vec element helpers that directly perform typed load/store through MemPtr<T> without requiring T: Copy. Even after public Vec operations are Copy-gated, direct import of vec/raw can still shallow-read or overwrite non-Copy payloads without initialized-cell/drop proof.

## 影響

Unsupported non-Copy Vec payloads can bypass the safer public API boundary, making Resource IR cell state and owner/drop obligations depend on caller discipline. This leaves a raw memory escape in Stage D collection safety work.

## 修正方針

Restrict vec_read_at and vec_write_at to T: Copy until OwnedBuffer<T> plus initialized cell state can express move-out and overwrite obligations. Add compile-fail doctests and source policy assertions that raw element helpers remain Copy-only.

## 検証

Run Vec source policy checks, focused raw element doctests, Vec doctests, issue check, and diff check.

## 修正内容

- `vec_read_at` / `vec_write_at` を `.T: Copy` に限定した。
- `raw/element.nepl` の doc comment に、non-Copy move-out / overwrite は `OwnedBuffer<T>` と initialized cell state transition が入るまで扱わない方針を明記した。
- `Vec<NonCopyPayload>` の raw read/write が `type.trait_bound.unsatisfied` で compile-fail になる doctest を追加した。
- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` の source policy を、raw element helper が Copy-only であり non-Copy doctest を持つことを監視する形に更新した。

## 検証結果

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_vec_borrowed_observers.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/raw/element.nepl --no-tree -o tmp/agent1-vec-raw-element-copy-bound-raw.json -j 1 --dist web/dist`: total=4, passed=4
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec --no-tree -o tmp/agent1-vec-raw-element-copy-bound-vec.json -j 4 --dist web/dist`: total=37, passed=37

## 親issueとの関係

- `ISS-20260425T000000Z-RV-STDLIB-004-91534828` の Stage D 残件のうち、raw element helper 経由で non-Copy payload API 境界を迂回できる入口を閉じた。
- full non-Copy collection support はこの issue では完了扱いにしない。non-Copy read/move/write/drop は `OwnedBuffer<T>`、initialized cell state、drop obligation、Resource IR lowering の上で別途設計する。
