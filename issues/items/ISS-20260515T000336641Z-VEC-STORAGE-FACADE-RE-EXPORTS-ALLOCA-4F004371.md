---
id: ISS-20260515T000336641Z-VEC-STORAGE-FACADE-RE-EXPORTS-ALLOCA-4F004371
title: "Vec storage facade re-exports allocation and cleanup internals"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-15
updated: 2026-05-15
target: "stdlib/alloc/collections/vec.nepl, stdlib/alloc/collections/vec/storage.nepl, stdlib/alloc/collections/vec/storage/alloc.nepl, stdlib/alloc/collections/vec/storage/api.nepl, stdlib/alloc/collections/vec/mutation/cleanup.nepl, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js, tests/stdlib/collection_cleanup_contract.n.md"
---

# ISS-20260515T000336641Z-VEC-STORAGE-FACADE-RE-EXPORTS-ALLOCA-4F004371: Vec storage facade re-exports allocation and cleanup internals

## 概要

The safe Vec root re-exports vec/storage, and vec/storage publicly merges allocation and cleanup implementation modules, so ordinary Vec imports expose vec_alloc_empty and vec_free_storage.

## 対象

- `stdlib/alloc/collections/vec.nepl`
- `stdlib/alloc/collections/vec/storage.nepl`
- `stdlib/alloc/collections/vec/storage/api.nepl`
- `stdlib/alloc/collections/vec/storage/alloc.nepl`
- `stdlib/alloc/collections/vec/storage/cleanup.nepl`
- `stdlib/alloc/collections/vec/mutation/cleanup.nepl`
- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `tests/stdlib/collection_cleanup_contract.n.md`

## 根拠

- `alloc/collections/vec` root は safe facade と説明しているが、`pub #import "./vec/storage" as @merge` を通じて `vec/storage` の公開面をそのまま通常 import 面へ出していた。
- `vec/storage` は `storage/alloc` と `storage/cleanup` を public merge していたため、root import だけで `vec_alloc_empty` と `vec_free_storage` に到達できた。
- `vec_alloc_empty` は internal allocation helper、`vec_free_storage` は element drop traversal を行わない storage-only cleanup helper であり、Stage 6 では public constructor / public free API と分けて扱うべきである。

## 問題

The safe Vec root re-exports vec/storage, and vec/storage publicly merges allocation and cleanup implementation modules, so ordinary Vec imports expose vec_alloc_empty and vec_free_storage.

## 影響

Callers can depend on storage allocation and storage-only cleanup helpers from the normal Vec facade, widening the Stage 6 safe surface and weakening reviewability of owner-token and initialized-cell boundaries.

## 修正方針

Split public Vec storage constructors into a storage/api facade and stop re-exporting storage/alloc and storage/cleanup through vec/storage. Keep implementation modules importing internal helpers explicitly, and add source policy plus compile-fail doctests proving root imports cannot see the helpers.

## 検証

Run focused Vec source policy, Vec root/storage doctests, focused collection cleanup doctests, issues check, and diff check.

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画 Stage 6](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-6-stdlib--self-host-buffer-%E7%A7%BB%E8%A1%8C)

## 2026-05-15 Agent 1 解決

`vec/storage/api.nepl` を追加し、public constructor の `new` / `with_capacity` を `storage/alloc` から移した。`storage/api` は `alloc::vec_alloc_empty` へ明示的に委譲するだけで、typed allocation helper 自体は public root facade へ出さない。

`vec/storage.nepl` は `view` / `api` / `fill` の public API だけを re-export し、`storage/alloc` / `storage/cleanup` を再公開しないようにした。`Vec.free` 実装は storage-only cleanup が必要なため、`mutation/cleanup.nepl` から `../storage/cleanup` を明示 import する形へ変更した。

`alloc/collections/vec.nepl` の root doctest は、これまで `vec/storage/alloc` 由来の `core/result` や `core/math` の偶発 visibility に依存していたため、`core/result` / `core/math` / `core/option` を明示 import するように直した。

回帰として、`nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` に `storage/alloc` / `storage/cleanup` が public storage facade から再公開されないこと、`new` / `with_capacity` が `storage/api` にあることを固定した。さらに `collection_cleanup_contract.n.md` に、root import だけでは `vec_alloc_empty` / `vec_free_storage` が undefined になる compile-fail doctest を追加した。

検証:

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec.nepl -i stdlib/alloc/collections/vec/storage.nepl -i stdlib/alloc/collections/vec/storage/api.nepl -i stdlib/alloc/collections/vec/storage/alloc.nepl --no-tree -o tmp/agent1-vec-storage-facade-modules.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i tests/stdlib/collection_cleanup_contract.n.md --no-tree -o tmp/agent1-vec-storage-facade-cleanup.json -j 1 --dist web/dist --assert-io`
