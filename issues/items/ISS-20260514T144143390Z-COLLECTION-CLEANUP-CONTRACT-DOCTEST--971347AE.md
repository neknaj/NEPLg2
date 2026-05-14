---
id: ISS-20260514T144143390Z-COLLECTION-CLEANUP-CONTRACT-DOCTEST--971347AE
title: "collection cleanup contract doctest masks per-API regressions"
area: stdlib
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-14
updated: 2026-05-14
target: tests/stdlib/collection_cleanup_contract.n.md
---

# ISS-20260514T144143390Z-COLLECTION-CLEANUP-CONTRACT-DOCTEST--971347AE: collection cleanup contract doctest masks per-API regressions

## 概要

collection cleanup の non-Copy 拒否回帰テストが複数 collection と複数 API を 1 つの compile_fail に束ねており、いずれか 1 箇所だけ trait_bound.unsatisfied を出せば全体が成功してしまう。

## 対象

- `tests/stdlib/collection_cleanup_contract.n.md`

## 根拠

- `tests/stdlib/collection_cleanup_contract.n.md` は `Vec.clear` / `Vec.free` / Stack / Queue / Deque / RingBuffer / BTree / List の non-Copy 拒否を 1 つの compile-fail snippet にまとめていた。
- `compile_fail` は snippet 全体が指定 diagnostic を 1 回でも出せば成功するため、個別 API の Copy bound が欠落しても、別 API の error に隠れる。
- `RV-STDLIB-004` の現行段階では non-Copy payload collection は未完成であり、Copy-only 境界を個別に固定する必要がある。

## 問題

collection cleanup の non-Copy 拒否回帰テストが複数 collection と複数 API を 1 つの compile_fail に束ねており、いずれか 1 箇所だけ trait_bound.unsatisfied を出せば全体が成功してしまう。

## 影響

Vec.clear/free や Stack/Queue/Deque/RingBuffer/BinaryHeap/BTree/List/HashMap/BTreeMap の個別 API が non-Copy payload を再び受け入れても、別の API の失敗に隠れて見逃す可能性がある。

## 修正方針

collection family/API ごとに compile_fail doctest を分割し、それぞれが独立して type.trait_bound.unsatisfied を要求する契約テストへ直す。

## 検証

node nodesrc/tests.js -i tests/stdlib/collection_cleanup_contract.n.md --no-tree -o tmp/agent1-collection-cleanup-contract.json -j 1 --dist web/dist --assert-io

## 修正結果

- `tests/stdlib/collection_cleanup_contract.n.md` の大きな compile-fail block を、collection family / API ごとの独立 doctest に分割した。
- `Vec.clear` と `Vec.free` を別 doctest に分け、片方の trait bound regression がもう片方の diagnostic に隠れないようにした。
- Stack / Queue / Deque / RingBuffer / BinaryHeap / BTreeSet / BTreeMap key / BTreeMap value / List / HashMap value の free 境界を、それぞれ独立した `type.trait_bound.unsatisfied` 期待で固定した。

## 検証結果

- `node nodesrc/tests.js -i tests/stdlib/collection_cleanup_contract.n.md --no-tree -o tmp/agent1-collection-cleanup-contract.json -j 1 --dist web/dist --assert-io`: 12/12 pass
