---
id: ISS-20260518T200613054Z-BYTEBUF-DOCTEST-IMPORTS-RAW-INTERNAL-387EC456
title: "ByteBuf removed-helper doctest imports raw internal MemPtr helper"
area: stdlib
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-18
updated: 2026-05-18
target: "tests/stdlib/bytebuf_result.n.md, nodesrc/test_stdlib_io_bytebuf_owner_boundary.js"
---

# ISS-20260518T200613054Z-BYTEBUF-DOCTEST-IMPORTS-RAW-INTERNAL-387EC456: ByteBuf removed-helper doctest imports raw internal MemPtr helper

## 概要

`io_bytebuf_from_owned_ptr` が削除済みであることを確認する compile-fail doctest が、`core/mem/internal` を import して `mem_ptr_wrap 0 1` から `MemPtr<u8>` を作っていた。

## 対象

- `tests/stdlib/bytebuf_result.n.md`
- `nodesrc/test_stdlib_io_bytebuf_owner_boundary.js`

## 根拠

- Stage 6 では `MemPtr<T>` は non-owning pointer view であり、ordinary doctest が raw internal helper を使って ownership-forging fixture を作るべきではない。
- この doctest の目的は、削除済み public helper `io_bytebuf_from_owned_ptr` が `alloc/io` から解決できないことを確認することであり、raw `MemPtr` の構築は目的に必要ない。
- compiler 側は ordinary source の raw helper use を source proof で制限しているため、stdlib の canonical regression も raw helper import を正常な fixture として残すべきではない。

## 問題

削除済み helper の未定義性を検査するだけの compile-fail fixture が raw internal helper に依存すると、`ByteBuf` owner boundary の回帰テストが「raw `MemPtr` を作ってよい」という旧設計の前提を保持してしまう。これは `MemPtr = non-owning view` / `RegionToken・ByteBufStorage = free obligation owner` の分離を弱め、将来の raw boundary 強化時に fixture 側が不適切な権限を要求する原因になる。

## 影響

ByteBuf/ByteBuilder の owner boundary は本体 API では閉じていても、回帰テストが raw internal import に依存することで、source proof に基づく汎用的な静的検査ではなく doctest 特例で通す方向へ戻りやすくなる。

## 修正方針

compile-fail doctest は `alloc/io` だけを import し、`io_bytebuf_from_owned_ptr 0 1` が解決できないことだけを検査する。`core/mem` / `core/mem/internal` / `mem_ptr_wrap` は使わない。source policy はこの removed-helper fixture に raw memory module import や raw `MemPtr` construction が戻らないことを監視する。

## 解決

- `tests/stdlib/bytebuf_result.n.md` の `io_bytebuf_rejects_raw_memptr_ownership_forging` から `core/mem` / `core/mem/internal` import と `mem_ptr_wrap` を削除した。
- `nodesrc/test_stdlib_io_bytebuf_owner_boundary.js` に、同 compile-fail doctest が raw memory module import や `mem_ptr_wrap` を使わず、削除済み helper の未定義性だけを検査することを監視する policy を追加した。

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- [raw-memory-backed APIs parent issue](./ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84.md)
- [ByteBuf/ByteBuilder owner boundary issue](./ISS-20260517T034837136Z-BYTEBUF-PUBLIC-API-CAN-FORGE-OWNERSH-16F30AE5.md)

## 検証結果

- `node nodesrc/test_stdlib_io_bytebuf_owner_boundary.js`: passed
- `node nodesrc/tests.js -i tests/stdlib/bytebuf_result.n.md --no-tree -o tmp/agent1-bytebuf-result-no-raw-internal.json -j 1 --dist web/dist --assert-io`: total=7, passed=7
- `node nodesrc/issues.js check`: passed
