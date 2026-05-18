---
id: ISS-20260518T180200702Z-HASHMAP-AND-HASHSET-UPDATE-ERRORS-DI-12982990
title: "HashMap and HashSet update errors discard consumed owners"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "stdlib/alloc/collections/hashmap/**, stdlib/alloc/collections/hashset/**, nodesrc/test_stdlib_hashmap_storage_contract.js, nodesrc/test_stdlib_hashset_storage_contract.js"
---

# ISS-20260518T180200702Z-HASHMAP-AND-HASHSET-UPDATE-ERRORS-DI-12982990: HashMap and HashSet update errors discard consumed owners

## 概要

HashMap/HashSet rehash, insert, and remove still return Result<Collection, Diag> after consuming the collection owner. Rehash allocation failure and remove-missing branches free the old storage internally and return only Diag, so callers cannot choose cleanup or retry and the owner transfer is hidden in implementation discipline.

## 対象

- `stdlib/alloc/collections/hashmap/**, stdlib/alloc/collections/hashset/**, nodesrc/test_stdlib_hashmap_storage_contract.js, nodesrc/test_stdlib_hashset_storage_contract.js`

## 根拠

- `hashmap_rehash_to` / `hashset_rehash_to` の allocation failure branch が旧 storage を内部で閉じ、`Err(Diag)` だけを返していた。
- `remove` の missing key branch も消費済み owner を caller へ返さず、HashMap/HashSet だけが他の owner-preserving collection update と不整合だった。

## 問題

HashMap/HashSet rehash, insert, and remove still return Result<Collection, Diag> after consuming the collection owner. Rehash allocation failure and remove-missing branches free the old storage internally and return only Diag, so callers cannot choose cleanup or retry and the owner transfer is hidden in implementation discipline.

## 影響

This violates the Stage 6 owner-preserving fallible update contract and can hide ownership mistakes from Resource IR checks. It also keeps HashMap/HashSet inconsistent with Vec, BTree, BinaryHeap, Deque, Stack, Queue, RingBuffer, and List push/update error payloads.

## 修正方針

Introduce owner-bearing HashMapUpdateError and HashSetUpdateError payloads with diag and owner accessors. Change rehash/prepare/insert/remove to return those errors and return the consumed owner on failure instead of freeing it internally. Update doctests and source policies to reject Diag-only owner-consuming update errors.

## 検証

Run focused HashMap/HashSet source policies, stdlib HashMap/HashSet doctests, issue index/check, and diff checks.

## 対応結果

`HashMapUpdateError<K,V,H>` と `HashSetUpdateError<T,H>` を追加し、`diag` と消費済み collection owner を error payload として保持するようにした。

- `hashmap_rehash_to` / `hashset_rehash_to` は allocation failure 時に旧 storage を破棄せず、元の `HashMap` / `HashSet` owner を error payload に戻す。
- `hashmap_prepare_insert` / `hashset_prepare_insert`、`insert`、`remove` の戻り型を owner-bearing update error へ統一した。
- missing key の `remove` は storage を内部で解放せず、caller が `*_update_error_owner` で owner を回収できるようにした。
- owner-bearing `Result` は generic `ok` / `err` helper ではなく、直接 `Result<..., UpdateError>::Ok/Err` で構成するようにした。これにより inactive branch の owner payload を Resource IR が誤って materialize する回帰を避ける。
- doctest と source policy を更新し、Diag-only update error と generic helper 経由の owner-bearing result へ戻る退行を拒否する。

検証中に `tests/stdlib/selfhost_req.n.md::doctest#5` の既存 StringBuilder owner leak を確認したため、`ISS-20260518T184314600Z-SELFHOST-REQ-STRINGBUILDER-DOCTEST-L-220A7D5E` として分離した。

## 検証結果

- `node nodesrc/test_stdlib_hashmap_storage_contract.js`: passed
- `node nodesrc/test_stdlib_hashset_storage_contract.js`: passed
- `node nodesrc/test_stdlib_collection_cleanup_contract.js`: passed
- `node nodesrc/tests.js -i stdlib/tests/hashmap.n.md --no-tree -o tmp/agent1-hashmap-update-error.json -j 1 --dist web/dist --assert-io`: total=1, passed=1
- `node nodesrc/tests.js -i stdlib/tests/hashset.n.md --no-tree -o tmp/agent1-hashset-update-error.json -j 1 --dist web/dist --assert-io`: total=2, passed=2
- `node nodesrc/tests.js -i tests/stdlib/collections_diag.n.md --no-tree -o tmp/agent1-collections-diag-update-error.json -j 1 --dist web/dist --assert-io`: total=4, passed=4
- `node nodesrc/tests.js -i stdlib/tests/hashmap_str.n.md -i stdlib/tests/hashset_str.n.md -i tests/stdlib/hash_collection_rehash.n.md -i tests/stdlib/pipe_collections.n.md -i tests/stdlib/traits_hash.n.md -i stdlib/alloc/collections/hashmap.nepl --no-tree -o tmp/agent1-hash-update-error-affected-no-selfhost.json -j 1 --dist web/dist --assert-io`: total=25, passed=25
- `node nodesrc/tests.js -i stdlib/alloc/collections/hashmap/types.nepl -i stdlib/alloc/collections/hashmap/api.nepl -i stdlib/alloc/collections/hashmap/rehash.nepl -i stdlib/alloc/collections/hashset/types.nepl -i stdlib/alloc/collections/hashset/api.nepl -i stdlib/alloc/collections/hashset/rehash.nepl --no-tree -o tmp/agent1-hash-update-error-modules.json -j 1 --dist web/dist --assert-io`: total=29, passed=29
- `tests/stdlib/selfhost_req.n.md` 全体は既存の `test_req_string_builder` owner leak で total=6, passed=5, failed=1。HashMap/HashSet API 変更の failure ではないため別 issue に分離した。
