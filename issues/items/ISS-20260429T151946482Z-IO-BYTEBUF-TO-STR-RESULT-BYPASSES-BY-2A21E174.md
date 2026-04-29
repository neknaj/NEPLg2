---
id: ISS-20260429T151946482Z-IO-BYTEBUF-TO-STR-RESULT-BYPASSES-BY-2A21E174
title: "io_bytebuf_to_str_result bypasses ByteBuf free boundary on conversion failures"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "stdlib/alloc/io.nepl, nodesrc/test_stdlib_bytebuf_utf8_boundary.js"
---

# ISS-20260429T151946482Z-IO-BYTEBUF-TO-STR-RESULT-BYPASSES-BY-2A21E174: io_bytebuf_to_str_result bypasses ByteBuf free boundary on conversion failures

## 概要

io_bytebuf_to_str_result matched ByteBuf.ptr by value and deallocated the extracted MemPtr directly on invalid UTF-8 or string allocation failure. That bypassed the centralized io_bytebuf_free consumption boundary and regressed the ByteBuf UTF-8 source policy.

## 対象

- `stdlib/alloc/io.nepl, nodesrc/test_stdlib_bytebuf_utf8_boundary.js`

## 根拠

- `node nodesrc/test_stdlib_bytebuf_utf8_boundary.js` が `io_bytebuf_to_str_result must reject invalid UTF-8 as InvalidUtf8 and consume the buffer` で失敗した。
- `io_bytebuf_to_str_result` は `match get buf "ptr"` で `ByteBuf.ptr` を取り出し、invalid UTF-8 / `string_from_mem_unchecked_result` 失敗 / 成功の各 path で `dealloc_ptr<u8> data ...` を直接呼んでいた。
- `ByteBuf` は [静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 6 の方針に沿って `Option<MemPtr<u8>>` で空/所有 storage を区別する構造に移行済みである。変換側が raw `data` を個別 free すると、その構造的な消費境界が `io_bytebuf_free` から分散してしまう。

## 問題

io_bytebuf_to_str_result matched ByteBuf.ptr by value and deallocated the extracted MemPtr directly on invalid UTF-8 or string allocation failure. That bypassed the centralized io_bytebuf_free consumption boundary and regressed the ByteBuf UTF-8 source policy.

## 影響

ByteBuf-to-str conversion can drift away from the structural Option<MemPtr<u8>> owner contract, making invalid UTF-8/error paths harder for Resource IR and source policy to verify.

## 修正方針

Read length and data through borrowed ByteBuf views, then consume the original ByteBuf exactly once through io_bytebuf_free on every non-empty success and failure path.

## 修正内容

- `io_bytebuf_to_str_result` は `io_bytebuf_len_ref &buf` と `io_bytebuf_ptr_ref &buf` で検証用 view を取得するようにした。
- invalid UTF-8、string allocation failure、成功時のいずれも、raw pointer ではなく元の `ByteBuf` を `io_bytebuf_free buf` で消費するようにした。
- `Option::None` かつ `len != 0` の不正状態と、`Option::Some` だが raw address が無効な状態でも、戻る前に `io_bytebuf_free buf` を通すようにした。
- 実装コメントを、借用 view と `io_bytebuf_free` 消費境界の責務分割を説明する内容へ更新した。

## 検証

- `node nodesrc/test_stdlib_bytebuf_utf8_boundary.js`: passed
- `node nodesrc/test_stdlib_io_bytebuf_owner_boundary.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/io.nepl --no-tree -o tmp/alloc-io-bytebuf-free-boundary.json -j 1 --dist web/dist`: total=1, passed=1
- `node nodesrc/tests.js -i tests/stdlib/bytebuf_result.n.md --no-tree -o tmp/bytebuf-result-free-boundary.json -j 1 --dist web/dist`: total=6, passed=6
