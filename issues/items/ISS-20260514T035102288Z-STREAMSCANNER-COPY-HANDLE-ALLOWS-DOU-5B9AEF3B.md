---
id: ISS-20260514T035102288Z-STREAMSCANNER-COPY-HANDLE-ALLOWS-DOU-5B9AEF3B
title: "StreamScanner Copy handle allows double close of raw-backed scanner storage"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-14
target: "stdlib/std/streamio/scanner*.nepl, stdlib/kp/kpgraph.nepl, tests/stdlib/{streamio,stdin,kp,kp_i64}.n.md, nodesrc/test_stdlib_streamio_scanner_boundary.js"
---

# ISS-20260514T035102288Z-STREAMSCANNER-COPY-HANDLE-ALLOWS-DOU-5B9AEF3B: StreamScanner Copy handle allows double close of raw-backed scanner storage

## 概要

StreamScanner owns raw-backed header and ByteBuf storage but implemented Copy/Clone and read APIs took the handle by value. This made duplicate handles indistinguishable from the owner, so close could be called multiple times or an alias could be used after close without compiler evidence.

## 対象

- `stdlib/std/streamio/scanner*.nepl`
- `stdlib/kp/kpgraph.nepl`
- `tests/stdlib/{streamio,stdin,kp,kp_i64}.n.md`
- `nodesrc/test_stdlib_streamio_scanner_boundary.js`
- `nodesrc/test_stdlib_streamio_no_unsafe_unwraps.js`

## 根拠

- `StreamScanner` は header pointer だけを field に持つが、header は scanner buffer pointer / len / cursor を保持し、`close` が header と owned ByteBuf storage を解放する。
- `impl Copy for StreamScanner` / `impl Clone for StreamScanner` により、同じ header を指す複数の owner-like handle が生成可能だった。
- `read sc` / `skip sc` / `is_eof sc` が by-value で公開されていたため、cursor 操作と ownership 消費の区別が型に出ていなかった。

## 問題

StreamScanner owns raw-backed header and ByteBuf storage but its public API made the handle Copy/Clone and accepted by-value reads. The type system therefore could not distinguish borrowed cursor access from owner-consuming close.

## 影響

A raw-memory-backed stdlib public API relied on user discipline for memory safety, contradicting Stage 6: the compiler and API types could not prove that scanner header/buffer free obligation is consumed exactly once.

## 修正方針

Completed:

- Removed `Copy` / `Clone` impls from `StreamScanner`.
- Changed `skip_ws` / `is_eof` / `skip` / `scan_token_impl` / numeric scan implementations / `read` overloads to accept `&StreamScanner`.
- Kept `close <(StreamScanner)*>()>` owner-consuming.
- Updated stdlib/test callers to use `read &sc`.
- Added source policy checks that reject `Copy` / `Clone` reintroduction and by-value scanner read/cursor APIs.
- Added compile-fail doctest coverage that `read sc` is rejected.

## 検証

Verified:

- `node nodesrc/test_stdlib_streamio_scanner_boundary.js`: passed
- `node nodesrc/test_stdlib_streamio_no_unsafe_unwraps.js`: passed
- `node nodesrc/run_doctest.js -i stdlib/std/streamio/scanner.nepl -n 1 --assert-io --dist web/dist`: passed
- `node nodesrc/tests.js -i tests/stdlib/stdin.n.md --no-tree -o tmp/agent1-streamscanner-borrow-stdin-3.json -j 1 --dist web/dist --assert-io`: total=5, passed=5

Known separate finding discovered while running broader streamio/kp suites:

- `std/streamio/writer` defines `close(StreamWriter)` in `writer/state`, but the public writer facade does not export a root `close` overload. This leaves `|> close` unresolved for `StreamWriter` through `std/streamio`. It is separate from the scanner owner alias fix and should be tracked/fixed as its own issue.
