---
id: ISS-20260518T081055566Z-SHA256-DOCTEST-BLOCKED-BY-OWNER-AGGR-B2EE3B20
title: "sha256 doctest blocked by owner aggregate field access boundary"
area: stdlib
status: fixed
resolved: true
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

根本原因は、2 つの異なる境界が混ざっていたことにある。

- stdlib 実装側の `sha256/api.nepl` は compiler-owned source だが、`ctx.buffer` の dotted access では source capability scanner に構造化された owner aggregate field access 証拠が残らない。
- doctest 側の helper は通常利用者 source として扱われるため、`get e "error"` / `get e "ctx"` で owner-backed aggregate の内部 field を読むこと自体が設計上許されない。

`type.owner_aggregate.field_access_restricted` 自体は正しい検査なので、通常 source へ owner aggregate field access を開放してはいけない。修正では compiler 検査を緩めず、stdlib の実装 source と公開 API の責務を分ける。

## 影響

解決前は canonical hash doctest が focused regression として使えず、string / hash の安全性変更時に SHA256 経路を一緒に確認できなかった。

解決後は、通常 source が owner-backed aggregate field を直接読む経路を開かずに、SHA256 doctest を再び focused regression として使える。

## 修正方針

以下の方針で修正した。

- `sha256/api.nepl` の `Sha256.buffer` access は `core/field` の明示呼び出しに統一し、compiler-owned stdlib source の構造化証拠が見える形にする。
- `Sha256UpdateError` には `sha256_update_error_kind(&Sha256UpdateError) -> StdErrorKind` と `sha256_update_error_ctx(Sha256UpdateError) -> Sha256` を追加し、error kind の borrow read と state owner の消費回収を分離する。
- `stdlib/tests/hash.n.md` は通常 source として直接 field access せず、公開 accessor 経由で error kind と state owner を扱う。
- source policy に `ctx.buffer` direct access の禁止と accessor 存在確認を追加し、同じ regression を検出できるようにする。

## 検証

2026-05-18 Agent 1:

- `node nodesrc/test_stdlib_sha256_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i stdlib/tests/hash.n.md -i stdlib/alloc/hash/sha256.nepl --no-tree -o tmp/agent1-sha256-owner-aggregate-boundary-hash.json -j 1 --dist web/dist --assert-io`: total=1, passed=1
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/agent1-sha256-owner-aggregate-boundary-memory-safety.json -j 1 --dist web/dist --assert-io`: total=52, passed=52
