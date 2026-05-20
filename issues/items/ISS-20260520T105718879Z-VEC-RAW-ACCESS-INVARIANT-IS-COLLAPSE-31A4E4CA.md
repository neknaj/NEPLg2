---
id: ISS-20260520T105718879Z-VEC-RAW-ACCESS-INVARIANT-IS-COLLAPSE-31A4E4CA
title: "Vec raw access invariant is collapsed to bool instead of typed proof"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/alloc/collections/vec/invariant.nepl, stdlib/alloc/collections/vec/**"
---

# ISS-20260520T105718879Z-VEC-RAW-ACCESS-INVARIANT-IS-COLLAPSE-31A4E4CA: Vec raw access invariant is collapsed to bool instead of typed proof

## 概要

Vec current Copy-only invariant returns bool, so raw access callers branch on a collapsed truth value and lose the reason/evidence for len, initialized_len, cap, and storage-state correlation. This weakens exhaustive-match based maintenance and can let future invariant variants be ignored silently.

## 対象

- `stdlib/alloc/collections/vec/invariant.nepl, stdlib/alloc/collections/vec/**`

## 根拠

- `stdlib/alloc/collections/vec/invariant.nepl` の `vec_buffer_current_copy_invariant<T>` / `vec_current_copy_invariant<T>` は、`len` / `initialized_len` / `cap` / `VecStorage` の相関を検査していたが、結果を `bool` に潰していた。
- `get` / `replace` / `push` / `pop` / transform / query / sort family は raw access 前提を `if not invariant` で扱っており、failure reason が型に残らなかった。
- Stage 6 の方針では、状態や検査結果を数値・文字列・opaque bool に寄せず、enum と exhaustive `match` で静的検査プログラム自体の変更漏れを検出しやすくする必要がある。

## 問題

Vec current Copy-only invariant returns bool, so raw access callers branch on a collapsed truth value and lose the reason/evidence for len, initialized_len, cap, and storage-state correlation. This weakens exhaustive-match based maintenance and can let future invariant variants be ignored silently.

## 影響

Stage 6 raw memory boundaries depend on every caller proving the same OwnedBuffer invariant before raw MemPtr load/store. A bool helper makes the proof opaque and makes static-check code itself easier to regress.

## 修正方針

Introduce a typed VecCopyInvariant enum with explicit invalid reasons, make vec_buffer_current_copy_invariant and vec_current_copy_invariant return that enum, and migrate raw boundary callers to match Valid versus Invalid reason exhaustively before raw traversal.

## 検証

Run Vec source policy, Vec invariant/access/mutation/query/transform/sort focused doctests, issues check, and diff checks.

## 解決

`VecCopyInvariantInvalid` と `VecCopyInvariant` を追加し、current Copy-only `Vec` invariant の結果を `Valid | Invalid(reason)` の typed proof/refutation として返すようにした。

`data_mem_ptr`、`get`、`replace`、`push`、`pop`、`drop_last`、aggregate / predicate query、transform family、quick / heap / merge / simple sort family は、raw data view・raw load/store・range copy・sort scratch allocation の前に `VecCopyInvariant` を `match` する形へ移行した。invalid branch は従来通り no-op / neutral result / owner-preserving error payload へ落とすが、分岐は bool ではなく enum variant に対する exhaustive match になった。

`nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` は、invariant helper が bool を返さないこと、typed invalid reason enum を持つこと、主要 raw boundary caller が `VecCopyInvariant::Invalid` / `Valid` を match してから raw traversal に進むことを監視する。

## 検証結果

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_collection_cleanup_contract.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/invariant.nepl --no-tree --dist web/dist -o tmp/agent1-vec-invariant-enum.json -j 1 --assert-io`: 1/1 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/access/data.nepl --no-tree --dist web/dist -o tmp/agent1-vec-invariant-enum-data.json -j 1 --assert-io`: 2/2 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/mutation/push.nepl --no-tree --dist web/dist -o tmp/agent1-vec-invariant-enum-mutation.json -j 1 --assert-io`: 3/3 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/query/get.nepl --no-tree --dist web/dist -o tmp/agent1-vec-invariant-enum-query.json -j 1 --assert-io`: 1/1 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/query/aggregate.nepl --no-tree --dist web/dist -o tmp/agent1-vec-invariant-enum-aggregate.json -j 1 --assert-io`: 3/3 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/query/predicate.nepl --no-tree --dist web/dist -o tmp/agent1-vec-invariant-enum-predicate.json -j 1 --assert-io`: 3/3 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/transform/map.nepl --no-tree --dist web/dist -o tmp/agent1-vec-invariant-enum-map.json -j 1 --assert-io`: 1/1 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/transform/filter.nepl --no-tree --dist web/dist -o tmp/agent1-vec-invariant-enum-filter.json -j 1 --assert-io`: 7/7 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/transform/prefix.nepl --no-tree --dist web/dist -o tmp/agent1-vec-invariant-enum-prefix.json -j 1 --assert-io`: 2/2 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/sort.nepl --no-tree --dist web/dist -o tmp/agent1-vec-invariant-enum-sort.json -j 1 --assert-io`: 3/3 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec.nepl --no-tree --dist web/dist -o tmp/agent1-vec-invariant-enum-root.json -j 1 --assert-io`: 3/3 passed
