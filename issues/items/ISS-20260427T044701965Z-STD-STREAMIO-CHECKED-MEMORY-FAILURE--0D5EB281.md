---
id: ISS-20260427T044701965Z-STD-STREAMIO-CHECKED-MEMORY-FAILURE--0D5EB281
title: "std/streamio が checked memory failure を trap する"
area: stdlib
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/std/streamio.nepl, tests/stdlib/streamio.n.md, nodesrc/test_stdlib_streamio_no_unsafe_unwraps.js"
source: ISS-20260425T000000Z-RV-STDLIB-010-BF35FCBB
---

# ISS-20260427T044701965Z-STD-STREAMIO-CHECKED-MEMORY-FAILURE--0D5EB281: std/streamio が checked memory failure を trap する

## 概要

std/streamio は StreamScanner header と StreamWriter の文字列/ByteBuf copy 経路で checked memory helper の失敗を unwrap / unreachable に変換している。

## 対象

- `stdlib/std/streamio.nepl, tests/stdlib/streamio.n.md, nodesrc/test_stdlib_streamio_no_unsafe_unwraps.js`

## 根拠

stream_scanner_load_header / stream_scanner_store_header は header field pointer 計算や load/store_i32 失敗を unreachable にしている。append_str_impl / append_bytebuf_impl は load_u8 の Option を unwrap している。

## 問題

stream input/output は self-host CLI の基本経路であり、buffer/header の異常や memory pressure が発生したときに Result/既存 facade の sentinel へ戻らず trap する。

## 影響

self-host compiler の標準入力 scanner と output writer が、診断や graceful failure へ進む前に落ちる可能性があり、RV-STDLIB-010 の unsafe helper debt が残る。

## 修正方針

checked memory helper は match で受け、内部 Result 版 helperに集約する。既存 facade は公開 API 互換の sentinel に丸め、writer は実際に書けた byte 数だけ header len を更新する。source policy regression で unsafe unwrap の再導入を防ぐ。

## 解決内容

- `stream_scanner_load_header_result` / `stream_scanner_store_header_result` を追加し、scanner header の `load_i32` / `store_i32` 失敗を `Result` として扱うようにした。
- 既存の `stream_scanner_load_header` / `stream_scanner_store_header` は、公開 API 互換のために失敗を `0` / no-op へ丸める facade にした。
- `scanner_from_bytes` は header field 初期化を Result 版 helper へ切り替え、失敗時に `ByteBuf` と header を解放して `Err` を返すようにした。
- `push_u8_impl` は `store_u8` が成功した場合だけ `WriteLen` を進めるようにし、checked store failure で writer header だけが先に進む状態を避けた。
- `append_str_impl` / `append_bytebuf_impl` は `load_u8` を `match` し、`push_u8_impl` 経由で 1 byte ずつ追記する形へ変更した。これにより 4096 byte を超える入力でも flush 境界を越えて安全に出力する。
- 数値 writer helper も直接 buffer store ではなく digit helper から `push_u8_impl` へ集約し、checked store の一貫性を保った。
- `nodesrc/test_stdlib_streamio_no_unsafe_unwraps.js` を追加し、CI/source policy と `doc/testing.md` に登録した。

## 検証

- `node nodesrc/test_stdlib_streamio_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i stdlib/std/streamio.nepl --no-tree -o tmp/streamio-checked-memory-docs.json -j 1`: 1/1 passed
- `node nodesrc/tests.js -i tests/stdlib/streamio.n.md --no-tree -o tmp/streamio-checked-memory-focused.json -j 1`: 13/13 passed
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-streamio-checked-memory.json -j 4`: 305/305 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-streamio-checked-memory.json -j 4`: 418/418 passed
