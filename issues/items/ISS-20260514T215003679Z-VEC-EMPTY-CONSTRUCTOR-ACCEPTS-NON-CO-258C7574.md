---
id: ISS-20260514T215003679Z-VEC-EMPTY-CONSTRUCTOR-ACCEPTS-NON-CO-258C7574
title: "Vec empty constructor accepts non-Copy payload without drop contract"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-15
target: "stdlib/alloc/collections/vec/storage/view.nepl, tests/stdlib/collection_cleanup_contract.n.md, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js"
---

# ISS-20260514T215003679Z-VEC-EMPTY-CONSTRUCTOR-ACCEPTS-NON-CO-258C7574: Vec empty constructor accepts non-Copy payload without drop contract

## 概要

vec_empty<T> is a public zero-allocation constructor without a Copy bound. It can still create Vec<NonCopyPayload> even though every meaningful Vec cleanup/update path is intentionally Copy-only until OwnedBuffer<T> and initialized-prefix drop traversal exist.

## 対象

- `stdlib/alloc/collections/vec/storage/view.nepl, tests/stdlib/collection_cleanup_contract.n.md, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`

## 根拠

- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6 は、collection を Copy read / borrowed read / owned remove-pop / container drop に分け、raw-backed storage discipline を safe public API へ漏らさない方針である。
- `vec_empty<T>` は allocation を行わないが、root `alloc/collections/vec` から見える public constructor であり、`Vec<T> 0 0 VecStorageState::Empty vec_empty_region<T>` により `RegionToken<T>` sentinel を含む public `Vec<T>` owner aggregate を作る。
- `Vec.free` / `clear` / `push` / `pop` / raw element helper / allocation constructor は、`OwnedBuffer<T>` と initialized-prefix drop traversal が未完成のため Copy-only に閉じている。`vec_empty<T>` だけを generic に残すと、safe surface が `Vec<NonCopyPayload>` owner を作れてしまう。

## 問題

vec_empty<T> is a public zero-allocation constructor without a Copy bound. It can still create Vec<NonCopyPayload> even though every meaningful Vec cleanup/update path is intentionally Copy-only until OwnedBuffer<T> and initialized-prefix drop traversal exist.

## 影響

The safe Vec facade exposes an unsupported non-Copy collection owner type. Even if the Empty state does not allocate runtime storage, callers can observe and propagate a Vec<T> state that the current cleanup contract cannot close, weakening the collection drop contract and confusing later Resource IR assumptions.

## 修正方針

Restrict vec_empty<T> to T: Copy, update source policy and compile-fail regression, and record the change under the collection cleanup parent issue.

## 検証

Run Vec source policy, collection cleanup doctest, issue validation, and whitespace checks.

## 解決内容

- `vec_empty<T>` を `vec_empty<T: Copy>` に変更した。
- `vec_empty` の doc comment に、Empty state が runtime allocation を持たなくても `Vec<T>` owner aggregate を返すため現行設計では Copy-only に閉じる理由を記述した。
- `tests/stdlib/collection_cleanup_contract.n.md` に `vec_empty<CleanupPayload>` が `type.trait_bound.unsatisfied` で拒否される compile-fail regression を追加した。
- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` に、`vec_empty` の Copy-only signature と doc contract を監視する source policy を追加した。

## 関連

- 親: `ISS-20260425T000000Z-RV-STDLIB-004-91534828`
- 親: `ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84`
- 計画: `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6
