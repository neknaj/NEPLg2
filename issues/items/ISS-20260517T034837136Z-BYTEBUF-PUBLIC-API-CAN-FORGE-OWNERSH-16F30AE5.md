---
id: ISS-20260517T034837136Z-BYTEBUF-PUBLIC-API-CAN-FORGE-OWNERSH-16F30AE5
title: "ByteBuf/ByteBuilder public APIs can forge ownership from MemPtr"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-17
updated: 2026-05-18
target: "stdlib/alloc/io/bytebuf.nepl, stdlib/alloc/io/bytebuilder/types.nepl, nodesrc/test_stdlib_io_bytebuf_owner_boundary.js, tests/stdlib/bytebuf_result.n.md"
---

# ISS-20260517T034837136Z-BYTEBUF-PUBLIC-API-CAN-FORGE-OWNERSH-16F30AE5: ByteBuf/ByteBuilder public APIs can forge ownership from MemPtr

## 概要

`io_bytebuf_from_owned_ptr` and `byte_builder_from_owned_ptr` remained public enough to construct owning byte storage from a non-owning `MemPtr<u8>`, contradicting the RegionToken owner boundary.

## 対象

- `stdlib/alloc/io/bytebuf.nepl`
- `stdlib/alloc/io/bytebuilder/types.nepl`
- `nodesrc/test_stdlib_io_bytebuf_owner_boundary.js`
- `tests/stdlib/bytebuf_result.n.md`

## 根拠

- `ByteBuf` and `ByteBuilder` are owner-bearing handles backed by `RegionToken<u8>`.
- `MemPtr<u8>` is a non-owning pointer view under the static-check complexity reduction plan Stage 6.
- The removed helpers wrapped caller-provided `MemPtr<u8>` values with `region_new`, so ordinary code could manufacture a free obligation without a compiler-checked allocation/region owner.
- The existing result-path doctests forged huge ByteBuf values with `mem_ptr_wrap 0` and `io_bytebuf_from_owned_ptr`, masking the same design error in tests.

## 問題

`io_bytebuf_from_owned_ptr` and `byte_builder_from_owned_ptr` constructed owning handles from raw `MemPtr<u8>` values. This allowed public code and tests to bypass allocation/region proof and create fake owner values.

## 影響

Ordinary stdlib users can forge ByteBuf/ByteBuilder free obligations from raw pointer views, undermining Stage 6 MemPtr non-owning / RegionToken owner separation and Resource IR ownership proofs.

## 修正方針

Remove raw MemPtr ingestion helpers; ByteBuf/ByteBuilder ownership must be created only from RegionToken finalization or checked constructors. Convert unsafe doctest fixtures that relied on forged huge ByteBuf values into boundary regressions or safe result-path tests.

## 検証

- `nodesrc/test_stdlib_io_bytebuf_owner_boundary.js` must reject `io_bytebuf_from_owned_ptr`, `byte_builder_from_owned_ptr`, and local `region_new ptr ...` wrappers.
- `tests/stdlib/bytebuf_result.n.md` must cover invalid UTF-8 result paths using safe ByteBuilder construction.
- A compile-fail doctest must prove `io_bytebuf_from_owned_ptr` is not available from `alloc/io`.

## 2026-05-17 Agent 1 修正結果

- `io_bytebuf_from_owned_ptr` を削除した。
- `byte_builder_from_owned_ptr` を削除した。
- `nodesrc/test_stdlib_io_bytebuf_owner_boundary.js` に raw MemPtr ingestion helper と caller-provided `MemPtr` wrapper の再導入禁止を追加した。
- `tests/stdlib/bytebuf_result.n.md` は fake huge ByteBuf を作る OOM fixture をやめ、ByteBuilder 経由で作った invalid UTF-8 buffer の error propagation と、removed helper の compile-fail regression に置き換えた。

検証:

- `node nodesrc/test_stdlib_io_bytebuf_owner_boundary.js`: passed
- `node nodesrc/tests.js -i tests/stdlib/bytebuf_result.n.md --no-tree -o tmp/bytebuf-result-no-raw-owner-forge.json -j 1`: 7/7 passed
- `node nodesrc/tests.js -i stdlib/alloc/io/bytebuf.nepl --no-tree -o tmp/alloc-io-bytebuf-module-no-raw-owner-forge.json -j 1`: 1/1 passed
- `node nodesrc/tests.js -i stdlib/alloc/io/bytebuilder.nepl --no-tree -o tmp/alloc-io-bytebuilder-module-no-raw-owner-forge.json -j 1`: 1/1 passed
- `node nodesrc/issues.js check --dir issues`: passed
- `git diff --check`: passed

## 2026-05-18 Agent 1 追跡修正

`ISS-20260518T200613054Z-BYTEBUF-DOCTEST-IMPORTS-RAW-INTERNAL-387EC456` として、removed-helper compile-fail doctest に残っていた raw internal fixture 依存を分離して修正した。

`io_bytebuf_from_owned_ptr` の未定義性を確認するだけなら、`core/mem/internal` を import して `mem_ptr_wrap` から `MemPtr<u8>` を作る必要はない。修正後の doctest は `alloc/io` だけを import し、削除済み helper が解決できないことを直接確認する。`nodesrc/test_stdlib_io_bytebuf_owner_boundary.js` もこの fixture に raw memory module import や `mem_ptr_wrap` が戻らないことを監視する。
