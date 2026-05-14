---
id: ISS-20260514T051405052Z-STREAMSCANNER-HIDES-BUFFER-OWNER-BEH-0977B2E3
title: "StreamScanner hides buffer owner behind raw header MemPtr"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-14
target: "stdlib/std/streamio/scanner/state.nepl, stdlib/std/streamio/scanner.nepl, stdlib/std/streamio/scanner/cursor.nepl, stdlib/std/streamio/scanner/number"
---

# ISS-20260514T051405052Z-STREAMSCANNER-HIDES-BUFFER-OWNER-BEH-0977B2E3: StreamScanner hides buffer owner behind raw header MemPtr

## 概要

StreamScanner still stores a raw MemPtr header as its only state field. The header contains the buffer pointer, length, and cursor position, so the actual ByteBuf owner is hidden behind a raw address and the MemPtr owner-field migration keeps a StreamScanner.header exception.

## 対象

- `stdlib/std/streamio/scanner/state.nepl, stdlib/std/streamio/scanner.nepl, stdlib/std/streamio/scanner/cursor.nepl, stdlib/std/streamio/scanner/number`

## 根拠

- `stdlib/std/streamio/scanner/state.nepl` の旧 `StreamScanner.header <MemPtr<u8>>` は、input buffer owner、byte length、cursor position を 1 つの raw header に混在させていた。
- cursor position を raw `load_i32` / `store_i32` で扱う設計は、Resource IR の initialized cell 証明と衝突し、scanner state が raw memory discipline を直接持ち続ける原因になっていた。
- `nodesrc/test_stdlib_memptr_owner_field_policy.js` では `StreamScanner.header` を transitional allowlist に入れる必要があり、Stage 6 の MemPtr owner field migration の完了条件を阻害していた。

## 問題

StreamScanner still stores a raw MemPtr header as its only state field. The header contains the buffer pointer, length, and cursor position, so the actual ByteBuf owner is hidden behind a raw address and the MemPtr owner-field migration keeps a StreamScanner.header exception.

## 影響

Stage 6 cannot prove scanner buffer ownership from the source type structure, and future scanner changes can continue to treat a raw pointer header as the owner of both cursor and buffer storage.

## 修正方針

Make StreamScanner carry the input ByteBuf owner directly and keep only a small cursor storage boundary for mutable position. Route byte reads and token slicing through ByteBuf-based helper functions, then remove the StreamScanner.header MemPtr exception.

## 対応

- `StreamScanner` を `bytes <ByteBuf>` と `cursor <Vec<i32>>` を持つ owning handle に変更し、input buffer owner を source type structure へ直接出した。
- cursor は raw byte header ではなく 1 要素の typed `Vec<i32>` とし、読み書きは `vec::get<i32>` / `vec::replace<i32>` 経由にした。これにより scanner state から raw `load_i32` / `store_i32` を除去した。
- scanner root、cursor helper、integer / float parser は `stream_scanner_byte_at`、`stream_scanner_len`、`stream_scanner_load_pos`、`stream_scanner_store_pos` を使う形へ移し、token parser が raw pointer header を直接扱わない構造にした。
- `StreamScanner.header` の transitional MemPtr owner-field exception を削除し、policy baseline を 7 件から 6 件へ下げた。

## 検証

- `node nodesrc/test_stdlib_streamio_scanner_boundary.js`
- `node nodesrc/test_stdlib_streamio_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_memptr_owner_field_policy.js`
- `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 7 --assert-io --dist web/dist`
- `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 8 --assert-io --dist web/dist`
- `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 9 --assert-io --dist web/dist`
- `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 10 --assert-io --dist web/dist`
- `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 11 --assert-io --dist web/dist`
- `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 12 --assert-io --dist web/dist`
- `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 13 --assert-io --dist web/dist`
- `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 14 --assert-io --dist web/dist`
- `node nodesrc/run_doctest.js -i tests/stdlib/streamio.n.md -n 15 --assert-io --dist web/dist`
