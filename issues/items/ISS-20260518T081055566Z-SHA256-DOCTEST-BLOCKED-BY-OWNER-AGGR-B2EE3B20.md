---
id: ISS-20260518T081055566Z-SHA256-DOCTEST-BLOCKED-BY-OWNER-AGGR-B2EE3B20
title: "sha256 doctest blocked by owner aggregate field access boundary"
area: stdlib
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "stdlib/tests/hash.n.md, stdlib/alloc/hash/sha256/api.nepl, nepl-core/src/source_capability/owner_aggregate"
---

# ISS-20260518T081055566Z-SHA256-DOCTEST-BLOCKED-BY-OWNER-AGGR-B2EE3B20: sha256 doctest blocked by owner aggregate field access boundary

## 概要

`stdlib/tests/hash.n.md::doctest#1` が、`Sha256` の owner-backed aggregate field access で `type.owner_aggregate.field_access_restricted` になり compile できない。今回の `alloc/string/byte_index` witness 化とは独立した既存境界問題である。

## 対象

- `stdlib/tests/hash.n.md, stdlib/alloc/hash/sha256/api.nepl, nepl-core/src/source_capability/owner_aggregate`

## 根拠

- focused run `node nodesrc/tests.js -i stdlib\tests\string.n.md -i stdlib\tests\hash.n.md --no-tree -o tmp\agent1-string-byte-index-proof-string-hash.json -j 1 --dist web\dist --assert-io` で、string doctest 9 件は通過し、hash doctest 1 件だけが compile failure になった。
- 失敗箇所は `stdlib/alloc/hash/sha256/api.nepl` の `ctx.buffer` field access in `sha256_update` / `sha256_free` / `sha256_finalize` と、virtual entry 側の `get e "ctx"`。
- 既存 `ISS-20260507T050025343Z-SHA256-HASH-DOCTEST-FAILS-RESOURCE-I-A4EE25CE` は match payload shadowing による `resource.cell.uninit` の修正済み issue であり、今回の owner aggregate field boundary とは診断も原因も異なる。
- `type.owner_aggregate.field_access_restricted` 自体は ordinary source から owner-backed aggregate 内部 field を触れないようにする重要な静的検査であり、単純に緩めてはいけない。

## 問題

SHA-256 実装は input buffer owner を `Sha256` aggregate に保持するが、現在の compiler/source capability boundary では `sha256/api.nepl` の実装 source が `ctx.buffer` を読む権限を証明できていない。さらに doctest helper が `Sha256UpdateError.ctx` を `get e "ctx"` で取り出す経路も owner-backed aggregate field access として拒否される。

修正では、`Sha256` が owner-preserving public accessor/result API を持つべきか、compiler-owned stdlib implementation source の owner aggregate field proof が不足しているのかを切り分ける必要がある。通常 source で owner aggregate field access を許す方向には戻さない。

## 影響

The canonical hash doctest cannot be used as a focused regression while SHA256 owner aggregate access is rejected, and future hash/string changes may need to skip the suite even though artifact hashing depends on it.

## 修正方針

Investigate whether Sha256 should expose owner-preserving accessors or whether compiler-owned stdlib implementation source proof should cover this owner aggregate field access. Do not weaken the owner aggregate restriction for ordinary source.

## 検証

Run node nodesrc/tests.js -i stdlib/tests/hash.n.md -i stdlib/alloc/hash/sha256.nepl --no-tree --dist web/dist -j 1 --assert-io and keep owner aggregate ordinary-source memory safety regressions passing.
