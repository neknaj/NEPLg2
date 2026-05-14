---
id: ISS-20260514T045613682Z-STREAMWRITER-STORES-RAW-MEMPTR-OWNER-448F8E4F
title: "StreamWriter stores raw MemPtr owner instead of ByteBuilder owner boundary"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-14
target: "stdlib/std/streamio/writer/state.nepl, nodesrc/test_stdlib_memptr_owner_field_policy.js"
---

# ISS-20260514T045613682Z-STREAMWRITER-STORES-RAW-MEMPTR-OWNER-448F8E4F: StreamWriter stores raw MemPtr owner instead of ByteBuilder owner boundary

## 概要

StreamWriter keeps its owned output buffer directly as MemPtr<u8>, leaving a raw pointer owner field on a public stream state even though ByteBuilder already provides an owner-preserving byte storage boundary.

## 対象

- `stdlib/std/streamio/writer/state.nepl, nodesrc/test_stdlib_memptr_owner_field_policy.js`

## 根拠

- `stdlib/std/streamio/writer/state.nepl` の `StreamWriter` が `buf <MemPtr<u8>>` / `cap <i32>` / `write_len <i32>` を直接 field に持ち、writer state 自体が raw pointer storage owner に見える設計になっていた。
- `nodesrc/test_stdlib_memptr_owner_field_policy.js` には `StreamWriter.buf` が transitional MemPtr owner field として残っており、Stage 6 の raw-memory-backed public API migration が writer state で止まっていた。
- 既存の `alloc/io/bytebuilder` は pointer / capacity / length を 1 つの owned byte storage boundary として扱えるため、writer が `MemPtr` owner field を直接公開する必要はなかった。

## 問題

StreamWriter keeps its owned output buffer directly as MemPtr<u8>, leaving a raw pointer owner field on a public stream state even though ByteBuilder already provides an owner-preserving byte storage boundary.

## 影響

Stage 6 raw-memory-backed API migration keeps a transitional MemPtr owner exception for StreamWriter, and future stream writer changes can continue to treat MemPtr as storage owner instead of a non-owning projection.

## 修正方針

Move StreamWriter buffer ownership to ByteBuilder, keep flush/close/push operations consuming and returning StreamWriter, and remove the StreamWriter.buf entry from the MemPtr owner-field migration policy.

## 検証

Run focused streamio doctests and the stdlib MemPtr owner-field policy.

## 対応内容

- `StreamWriter` の state layout を direct `MemPtr` / `cap` / `write_len` field から `builder <ByteBuilder>` / `target <StreamWriterTargetKind>` へ変更した。
- `stream_writer_new` は `byte_builder_with_capacity 4096` で buffer owner を作り、writer state には `ByteBuilder` owner boundary だけを保持するようにした。
- `stream_writer_close_impl` は `ByteBuilder` を move して `byte_builder_free` に委譲するようにし、writer 独自の raw pointer free helper を削除した。
- `drain_impl` は `ByteBuilder.ptr` の `Option` view を通して stdout/stderr へ flush し、flush 後は `byte_builder_with_len builder 0 target` で同じ owner の pending length だけを戻す形にした。
- `push_u8_impl` は direct `store_u8` ではなく `byte_builder_push_u8` に storage write と length advance を委譲し、writer state から raw pointer mutation を追い出した。
- `nodesrc/test_stdlib_memptr_owner_field_policy.js` から `StreamWriter.buf` の transitional exception を削除し、streamio writer policy を `ByteBuilder` owner boundary 前提に更新した。

## 検証結果

- `node nodesrc/test_stdlib_memptr_owner_field_policy.js` は passing。transitional MemPtr owner field は 8 件から 7 件に減少。
- `node nodesrc/test_static_check_boundary_responsibility.js` は passing。
- `node nodesrc/test_stdlib_streamio_writer_boundary.js` は passing。
- `node nodesrc/test_stdlib_streamio_no_unsafe_unwraps.js` は passing。
- `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 1 --assert-io --dist web/dist` は passing。
- `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 2 --assert-io --dist web/dist` は passing。
- `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 4 --assert-io --dist web/dist` は passing。
- `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 5 --assert-io --dist web/dist` は passing。
- `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 111 --dist web/dist` は passing。`StreamWriter` の non-Copy / moved-cell rejection 境界を維持。
- `node nodesrc/run_source_policy_regressions.js` は passing。
- `node nodesrc/test_resource_gate_order.js` は passing。
- `node nodesrc/test_resource_checker_responsibility.js` は passing。
