---
id: ISS-20260514T171944501Z-BYTEBUF-AND-BYTEBUILDER-EXPOSE-EMPTY-6E06A830
title: "ByteBuf and ByteBuilder expose empty RegionToken sentinel helpers"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-15
target: "stdlib/alloc/io/bytebuilder/types.nepl, stdlib/alloc/io/bytebuf.nepl, nodesrc/test_stdlib_io_bytebuf_owner_boundary.js, nodesrc/source_policy/stdlib_builder_owner.js"
---

# ISS-20260514T171944501Z-BYTEBUF-AND-BYTEBUILDER-EXPOSE-EMPTY-6E06A830: ByteBuf and ByteBuilder expose empty RegionToken sentinel helpers

## 概要

ByteBuilder and ByteBuf still publish zero-size RegionToken sentinel constructors as public helpers. Vec already made the equivalent vec_empty_region private because the sentinel exists only to satisfy the transitional RegionToken-backed layout and must not become an external owner-token API.

## 対象

- `stdlib/alloc/io/bytebuilder/types.nepl, stdlib/alloc/io/bytebuf.nepl, nodesrc/test_stdlib_io_bytebuf_owner_boundary.js, nodesrc/source_policy/stdlib_builder_owner.js`

## 根拠

- `Vec` の同種 helper `vec_empty_region<T>` は、`ISS-20260514T155620178Z-VEC-EMPTY-REGIONTOKEN-SENTINEL-HELPE-B3CF72E9` で private 化済みである。
- `byte_builder_empty_region` / `io_bytebuf_empty_region` は zero-size `RegionToken<u8>` sentinel を作るだけで、public caller が直接扱うべき typed value ではない。
- `byte_builder_empty` / `io_bytebuf_empty` が typed empty handle を返すため、sentinel helper を public re-export する必要はない。

## 問題

ByteBuilder and ByteBuf still publish zero-size RegionToken sentinel constructors as public helpers. Vec already made the equivalent vec_empty_region private because the sentinel exists only to satisfy the transitional RegionToken-backed layout and must not become an external owner-token API.

## 影響

External source can depend on transitional empty RegionToken construction instead of the typed empty ByteBuilder/ByteBuf constructors. That keeps Stage 6 raw-memory-backed API migration exposed to implementation detail and weakens the intended separation between public safe handles and internal owner-token layout.

## 修正方針

Make byte_builder_empty_region and io_bytebuf_empty_region private implementation helpers, keep byte_builder_empty and io_bytebuf_empty as the public typed constructors, and extend source policy checks so sentinel helpers cannot be made public again.

## 検証

Run ByteBuilder/ByteBuf owner-boundary source policies, focused byte builder / bytebuf doctests, issue validation, and diff whitespace checks.

## 解決内容

`byte_builder_empty_region` と `io_bytebuf_empty_region` を private helper にした。公開 API は `byte_builder_empty -> ByteBuilder` と `io_bytebuf_empty -> ByteBuf` に限定し、zero-size `RegionToken` sentinel は各 implementation file の内部 detail としてだけ使う。

`nodesrc/source_policy/stdlib_builder_owner.js` と `nodesrc/test_stdlib_io_bytebuf_owner_boundary.js` に、empty sentinel helper が `pub fn` へ戻らないことと typed empty constructor が public のまま残ることを追加した。`tests/stdlib/memory_safety.n.md` には両 helper が facade import から解決できない compile-fail regression を追加した。

## 関連

- Parent: `ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84`
- Doc: `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6
