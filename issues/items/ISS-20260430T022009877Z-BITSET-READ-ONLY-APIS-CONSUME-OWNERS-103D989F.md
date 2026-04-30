---
id: ISS-20260430T022009877Z-BITSET-READ-ONLY-APIS-CONSUME-OWNERS-103D989F
title: "BitSet read-only APIs consume owners by value instead of borrowing"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-30
updated: 2026-04-30
target: stdlib/alloc/collections/bitset.nepl
---

# ISS-20260430T022009877Z-BITSET-READ-ONLY-APIS-CONSUME-OWNERS-103D989F: BitSet read-only APIs consume owners by value instead of borrowing

## 概要

BitSet len/contains style read-only APIs take BitSet by value, so callers that only inspect the collection either move the owner and cannot free it afterwards, or leave an owner leak when tests end after the read. This conflicts with the memory-safety policy and makes static checks report real leaks in otherwise read-only code.

## 対象

- `stdlib/alloc/collections/bitset.nepl`

## 根拠

- `stdlib/alloc/collections/bitset.nepl` の `len` は `fn len <(BitSet)->i32>`、`contains` は `fn contains <(BitSet,i32)*>Result<bool, Diag>>` で、読み取り専用なのに `BitSet` owner を値で受け取っていた。
- `stdlib/tests/bitset.n.md` と `tests/stdlib/bitset_collections.n.md` は、`contains bs0` / `len bs2` のように observer ごとに owner を消費するため、同じ `BitSet` を読み取った後に `free` する形の回帰テストになっていなかった。
- sibling review で、同じ owner-consuming observer pattern が AdjacencyMatrix / BloomFilter / CountingBloomFilter / Fenwick / SparseSet にも残っていることを確認し、個別 issue を追加した。

## 問題

BitSet len/contains style read-only APIs take BitSet by value, so callers that only inspect the collection either move the owner and cannot free it afterwards, or leave an owner leak when tests end after the read. This conflicts with the memory-safety policy and makes static checks report real leaks in otherwise read-only code.

## 影響

Collection users must choose between by-value observer calls that consume ownership and ad hoc field reads that bypass the public API. Self-host stdlib code will either leak owned buffers or encode unnatural workarounds instead of using checked borrowed observers.

## 修正方針

Redesign BitSet observer APIs around borrowed receivers such as &BitSet, update examples/tests to call borrowed observers, and review sibling collection observer APIs for the same by-value owner pattern.

## 検証

Add compile/run tests that read BitSet length and membership through borrowed APIs, then explicitly free the same BitSet without move-check or owner-check diagnostics.

確認済み:

- `node nodesrc/tests.js -i stdlib/tests/bitset.n.md --no-tree -o tmp/bitset-stdlib-borrowed-observers.json -j 1` (`total=2`, `passed=2`, `failed=0`)
- `node nodesrc/tests.js -i tests/stdlib/bitset_collections.n.md --no-tree -o tmp/bitset-collections-borrowed-observers.json -j 1` (`total=2`, `passed=2`, `failed=0`)
- `node nodesrc/tests.js -i stdlib/alloc/collections/bitset.nepl --no-tree -o tmp/bitset-doctest-borrowed-observers.json -j 1` (`total=7`, `passed=7`, `failed=0`)
- `node nodesrc/test_stdlib_bitset_borrowed_observers.js`: passed
- `node nodesrc/test_stdlib_bitset_no_unsafe_unwraps.js`: passed
- `cargo test -p nepl-core --test pipe_operator -- pipe_complete_overloaded_source_call_into_target --nocapture`: passed

## 修正内容

- `BitSet.len` と `BitSet.contains` を `&BitSet` receiver に変更し、読み取りで owner を移動しない公開 API にした。
- BitSet doctest / `.n.md` tests を、1 つの `BitSet` に対して複数回 borrowed observer を呼び、その後 `free` する形に直した。
- `nodesrc/test_stdlib_bitset_borrowed_observers.js` を追加し、by-value observer signature と by-value test usage が戻らないよう source policy に登録した。
