---
id: ISS-20260429T191522911Z-DIAG-BY-VALUE-OBSERVERS-LEAVE-OWNED--9A8FBE5F
title: "Diag by-value observers leave owned message payloads"
area: stdlib
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "stdlib/alloc/diag/error.nepl, tests/stdlib/collections_diag.n.md"
---

# ISS-20260429T191522911Z-DIAG-BY-VALUE-OBSERVERS-LEAVE-OWNED--9A8FBE5F: Diag by-value observers leave owned message payloads

## 概要

Diag is documented and implemented as Copy, but it carries str/Option<str> payload fields. Matching Result::Err d and observing the diagnostic kind through diag_std_error_kind_str leaves d.message/notes/help/source ownership obligations live under strict ResourceIR checks.

## 対象

- `stdlib/alloc/diag/error.nepl, tests/stdlib/collections_diag.n.md`

## 根拠

- `node nodesrc/tests.js -i stdlib/alloc/collections/hashmap.nepl -i stdlib/tests/hashmap.n.md -i stdlib/tests/hashmap_str.n.md -i tests/stdlib/hash_collection_rehash.n.md -i tests/stdlib/pipe_collections.n.md -i tests/stdlib/traits_hash.n.md -i tests/stdlib/selfhost_req.n.md -i tests/stdlib/collections_diag.n.md --no-tree -o tmp/hashmap-owner-contract-after-trunk.json -j 1 --dist web/dist` の `tests/stdlib/collections_diag.n.md::doctest#1` が、`Result::Err d` branch の `Local("d").Field(index=1, offset=20)` owner obligation leak で失敗する。
- 同じ run で HashMap 本体、HashMap grow、HashMap pipe/selfhost/trait fixture は pass しており、残件は HashMap storage owner contract ではなく `Diag` payload の所有契約に分離できる。
- `Diag` は `Copy` impl を持つ一方で、`message`, `notes`, `help`, `source` に `str` / `Option<str>` を保持している。`diag_std_error_kind_str` の by-value overload は kind だけを読んで戻るため、payload owner を消費・解放しない。

## 問題

Diag is documented and implemented as Copy, but it carries str/Option<str> payload fields. Matching Result::Err d and observing the diagnostic kind through diag_std_error_kind_str leaves d.message/notes/help/source ownership obligations live under strict ResourceIR checks.

## 影響

Collection diagnostic fixtures cannot inspect Err(Diag) without either leaking diagnostic payload owners or weakening ResourceIR ownership checks. This also makes future self-host diagnostic handling unsafe because diagnostics can appear Copy while still carrying owned string payloads.

## 修正方針

Redesign Diag ownership so diagnostic payload fields are either non-owning/static by construction or have an explicit consume/free/drop path. Update by-value observer overloads to consume or avoid owned payload obligations, and add regression coverage for Result::Err(Diag) inspection.

## 検証

Run tests/stdlib/collections_diag.n.md::doctest#1 and stdlib/alloc/diag/error.nepl diagnostics tests under strict ResourceIR after the redesign.
