---
id: ISS-20260426T060223863Z-BYTEBUF-CONVERSIONS-HIDE-ALLOCATION--3BF03711
title: "ByteBuf conversions hide allocation failure as empty values"
area: stdlib
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-26
updated: 2026-04-26
target: stdlib/alloc/io.nepl
---

# ISS-20260426T060223863Z-BYTEBUF-CONVERSIONS-HIDE-ALLOCATION--3BF03711: ByteBuf conversions hide allocation failure as empty values

## 概要

io_bytebuf_from_str は alloc_ptr 失敗時に io_bytebuf_empty を返し、io_bytebuf_to_str は string allocation 失敗時に ByteBuf を解放して空文字列を返す。どちらも API が Result を返さないため、空入力と allocation failure を呼び出し側が区別できない。

## 対象

- `stdlib/alloc/io.nepl`

## 根拠

- `io_bytebuf_from_str` は `alloc_ptr<u8>` 失敗時に `io_bytebuf_empty` を返していたため、空入力と確保失敗を区別できなかった。
- `io_bytebuf_to_str` は `string_alloc_region` 失敗時に入力 `ByteBuf` を解放した上で空文字列を返していたため、空 buffer と確保失敗を区別できなかった。
- `std/streamio`、`std/io`、`std/fs` の上位 facade もこれらの非 Result helper を使っており、Result を返す API 面でも失敗を成功値へ潰し得た。

## 問題

io_bytebuf_from_str は alloc_ptr 失敗時に io_bytebuf_empty を返し、io_bytebuf_to_str は string allocation 失敗時に ByteBuf を解放して空文字列を返す。どちらも API が Result を返さないため、空入力と allocation failure を呼び出し側が区別できない。

## 影響

binary I/O や self-host artifact generation で allocation failure が正常な空 payload として扱われ、出力欠落や破損を検出できない。Result を値として扱う stdlib 方針とも不整合になる。

## 修正方針

Result-returning variants を追加し、既存 helper は安全な wrapper に移行する。ByteBuf の所有権、失敗時解放、空入力を区別する doctest を追加する。

## 検証

alloc/io と streamio/io の ByteBuf conversion tests を追加し、空文字列と allocation failure 設計を分離して検証する。

## 対応結果

- `io_bytebuf_from_str_result` と `io_bytebuf_to_str_result` を追加し、allocation failure を `StdErrorKind::OutOfMemory` として返すようにした。
- 既存の `io_bytebuf_from_str` / `io_bytebuf_to_str` は互換 facade として残し、失敗時 fallback を明示した。
- `stream_bytes_from_str_result` / `stream_bytes_to_str_result` と `fs_bytes_to_string_result` を追加した。
- `std/streamio`、`std/io`、`std/fs` の Result-returning read / conversion 経路を Result variant へ接続し、上位 API が allocation failure を成功値へ潰さないようにした。
- `tests/stdlib/bytebuf_result.n.md` を追加し、roundtrip、空 buffer、allocation failure、`std/io` / `std/streamio` / `std/fs` facade の伝播を固定した。

## 確認結果

- `node nodesrc/tests.js -i tests/stdlib/bytebuf_result.n.md --no-tree -o tmp/bytebuf-result-focused-4.json -j 1`: `total=6`, `passed=6`, `failed=0`
- `node nodesrc/tests.js -i stdlib/alloc/io.nepl -i stdlib/std/streamio.nepl -i stdlib/std/io.nepl -i stdlib/std/fs.nepl -i tests/stdlib/bytebuf_result.n.md -i tests/stdlib/streamio.n.md -i tests/stdlib/io.n.md -i tests/stdlib/fs.n.md --no-tree -o tmp/bytebuf-result-suite-2.json -j 2`: `total=35`, `passed=35`, `failed=0`
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/bytebuf-result-stdlib-full-2.json -j 4`: `total=404`, `passed=404`, `failed=0`
