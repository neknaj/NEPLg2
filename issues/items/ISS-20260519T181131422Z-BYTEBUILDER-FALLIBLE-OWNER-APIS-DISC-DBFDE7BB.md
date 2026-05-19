---
id: ISS-20260519T181131422Z-BYTEBUILDER-FALLIBLE-OWNER-APIS-DISC-DBFDE7BB
title: "ByteBuilder fallible owner APIs discard builder owner"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-19
updated: 2026-05-20
target: stdlib/alloc/io/bytebuilder
---

# ISS-20260519T181131422Z-BYTEBUILDER-FALLIBLE-OWNER-APIS-DISC-DBFDE7BB: ByteBuilder fallible owner APIs discard builder owner

## 概要

ByteBuilder reserve, append, and finish consume ByteBuilder owner but return bare StdErrorKind after freeing or dropping storage on failure. The failure-path owner transfer is hidden in implementation discipline instead of the API type.

## 対象

- `stdlib/alloc/io/bytebuilder`

## 根拠

- `byte_builder_reserve` / `byte_builder_push_*` / `byte_builder_finish` は `ByteBuilder` owner を値渡しで消費するにもかかわらず、旧 API では `Result<ByteBuilder, StdErrorKind>` または `Result<ByteBuf, StdErrorKind>` を返していた。
- grow / append / finish failure では実装内で `byte_builder_free` や realloc failure cleanup を行い、caller が cleanup / retry を選ぶための owner payload が型に現れなかった。
- Stage 6 の List / Deque / BinaryHeap などは owner-consuming fallible update の Err payload に元 owner を戻す方向へ揃えており、ByteBuilder だけ bare error contract を残すと byte buffer owner boundary が不整合になる。

## 問題

ByteBuilder reserve, append, and finish consume ByteBuilder owner but return bare StdErrorKind after freeing or dropping storage on failure. The failure-path owner transfer is hidden in implementation discipline instead of the API type.

## 影響

Stage 6 raw-memory-backed APIs stay inconsistent with owner-preserving Vec/List update contracts, and Resource IR/source policy cannot prove caller cleanup or retry obligations from the type signature.

## 修正方針

Introduce owner-preserving ByteBuilder error payloads for update and finish failures, keep realloc failure owners instead of freeing them, update StringBuilder and tests to consume the new typed errors, and add source policy coverage rejecting bare-error owner-consuming ByteBuilder APIs.

## 検証

Run focused ByteBuilder/StringBuilder doctests and source policy regressions.

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- [stdlib collection / mem / string static safety design](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md)

## 解決内容

2026-05-20 に修正した。`ByteBuilderError` は `builder: ByteBuilder` と `error: StdErrorKind` を持ち、`byte_builder_reserve`、byte append 系、`byte_builder_finish` は失敗時に入力 builder owner をこの payload で caller へ戻す。

`byte_builder_push_bytebuf` は入力 builder と入力 `ByteBuf` の 2 owner を消費するため、`ByteBuilderByteBufError` を導入して両 owner と error kind を保持する。成功時だけ `ByteBuf` は実装内で閉じ、失敗時は caller が cleanup / retry を決められる。

realloc helper は `byte_builder_realloc_region_or_keep` とし、失敗時に旧 `RegionToken<u8>` を `RegionReallocError<u8>` 経由で返す。ByteBuilder 側で failure cleanup を隠さず、owned storage branch を `ByteBuilderError` に再構成する。

既存の `StringBuilder` / `StreamWriter` public API は今回の対象外として維持したが、内部で受け取る `ByteBuilderError` は必ず kind を取り出した後に `byte_builder_error_free` で閉じるようにした。これにより public API の互換ではなく、既存 wrapper が内部 owner payload を隠して cleanup する責務を明示した。

## 対応 stage

- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6: raw-memory-backed stdlib API の owner-consuming fallible update を owner-preserving error payload へ揃える作業。

## 検証結果

- `node nodesrc/test_stdlib_builder_owner_boundary.js`: passed
- `node nodesrc/test_stdlib_streamio_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i tests/stdlib/byte_builder.n.md --no-tree -o tmp/agent1-byte-builder-owner-errors.json -j 1 --dist web/dist --assert-io`: total=3, passed=3
- `node nodesrc/tests.js -i tests/stdlib/bytebuf_result.n.md --no-tree -o tmp/agent1-bytebuf-result-bytebuilder-errors.json -j 1 --dist web/dist --assert-io`: total=7, passed=7
- `node nodesrc/tests.js -i tests/stdlib/string_char.n.md --no-tree -o tmp/agent1-string-char-bytebuilder-errors.json -j 1 --dist web/dist --assert-io`: total=3, passed=3
- `node nodesrc/tests.js -i stdlib/alloc/io/bytebuilder/types.nepl -i stdlib/alloc/io/bytebuilder/storage.nepl -i stdlib/alloc/io/bytebuilder/append.nepl -i stdlib/alloc/io/bytebuilder/build.nepl --no-tree -o tmp/agent1-bytebuilder-docs-after-error-docs.json -j 1 --dist web/dist --assert-io`: total=12, passed=12
- `node nodesrc/tests.js -i tests/stdlib/string.n.md --no-tree -o tmp/agent1-string-bytebuilder-errors.json -j 1 --dist web/dist --assert-io`: total=17, passed=17
- `node nodesrc/tests.js -i tests/stdlib/streamio.n.md --no-tree -o tmp/agent1-streamio-bytebuilder-errors.json -j 1 --dist web/dist --assert-io`: total=16, passed=16
- `node nodesrc/tests.js -i tests/stdlib/text_utf8.n.md --no-tree -o tmp/agent1-text-utf8-bytebuilder-errors.json -j 1 --dist web/dist --assert-io`: total=9, passed=9
