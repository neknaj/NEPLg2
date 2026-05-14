---
id: ISS-20260514T041207146Z-STREAMWRITER-CLOSE-IS-NOT-EXPORTED-T-ED15EF36
title: "StreamWriter close is not exported through streamio writer facade"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-14
target: "stdlib/std/streamio/writer.nepl, stdlib/std/streamio/writer/state.nepl, tests/stdlib/{streamio,kp,kp_i64}.n.md"
---

# ISS-20260514T041207146Z-STREAMWRITER-CLOSE-IS-NOT-EXPORTED-T-ED15EF36: StreamWriter close is not exported through streamio writer facade

## 概要

StreamWriter close is defined in std/streamio/writer/state, but std/streamio/writer does not expose a root public close overload. Users importing std/streamio or std/streamio/writer cannot resolve |> close for StreamWriter even though write/flush are public facade APIs.

## 対象

- `stdlib/std/streamio/writer.nepl, stdlib/std/streamio/writer/state.nepl, tests/stdlib/{streamio,kp,kp_i64}.n.md`

## 根拠

- `node nodesrc/run_doctest.js -i stdlib/std/streamio/writer.nepl -n 1 --assert-io --dist web/dist` が修正前に `close write w "x"` の overload を解決できず compile fail した。
- `node nodesrc/tests.js -i tests/stdlib/streamio.n.md --no-tree -o tmp/agent1-streamwriter-close-streamio.json -j 1 --dist web/dist --assert-io` は修正前に `|> close` を使う 8 case が `type.overload.no_match` で失敗していた。
- state module だけに `close(StreamWriter)` が存在すると、root facade が所有する `write` / `flush` と public cleanup API が分断され、`std/streamio` import 経由で owner-consuming cleanup を呼べない。

## 問題

StreamWriter close is defined in std/streamio/writer/state, but std/streamio/writer does not expose a root public close overload. Users importing std/streamio or std/streamio/writer cannot resolve |> close for StreamWriter even though write/flush are public facade APIs.

## 影響

streamio and kp doctests that use buffered writer cannot compile. The public writer API is incomplete and ownership cleanup for StreamWriter is not available through the facade boundary.

## 修正方針

Move/rename the internal state cleanup to an explicit implementation helper and expose a root pub close overload in std/streamio/writer that consumes StreamWriter. Add source/doctest coverage that open/write/flush/close all resolve through the facade.

## 検証

- `stdlib/std/streamio/writer/state.nepl` の cleanup を `stream_writer_close_impl(StreamWriter)` に改名し、state layout を知る実装 helper に限定した。
- `stdlib/std/streamio/writer.nepl` に public `close(StreamWriter)` facade を追加し、`StreamWriter` owner を消費して `stream_writer_close_impl` に委譲するようにした。
- `nodesrc/test_stdlib_streamio_writer_boundary.js` と `nodesrc/test_stdlib_streamio_no_unsafe_unwraps.js` に、root が public close を所有し、state が common-name close を持たないことを固定する source policy を追加した。
- `node nodesrc/test_stdlib_streamio_writer_boundary.js`: passed
- `node nodesrc/test_stdlib_streamio_no_unsafe_unwraps.js`: passed
- `node nodesrc/run_doctest.js -i stdlib/std/streamio/writer.nepl -n 1 --assert-io --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i stdlib/std/streamio.nepl -n 1 --assert-io --dist web/dist`: passed
- `node nodesrc/tests.js -i tests/stdlib/streamio.n.md --no-tree -o tmp/agent1-streamwriter-close-streamio.json -j 1 --dist web/dist --assert-io`: total=15, passed=15
- `node nodesrc/tests.js -i tests/stdlib/kp_i64.n.md --no-tree -o tmp/agent1-streamwriter-close-kp-i64.json -j 1 --dist web/dist --assert-io`: total=4, passed=4
- `node nodesrc/tests.js -i tests/stdlib/kp.n.md --no-tree -o tmp/agent1-streamwriter-close-kp.json -j 1 --dist web/dist --assert-io`: total=7, passed=4, failed=3。残りは StreamWriter close とは別に、`StreamScanner` owner close 欠落と `core/mem/allocator` import 欠落の stale fixture 問題として `ISS-20260514T042244722Z-KP-FOCUSED-DOCTESTS-ARE-STALE-AFTER--DAB8C87D` に分離した。
