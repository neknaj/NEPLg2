---
id: ISS-20260519T064344008Z-STREAMSCANNER-TOKEN-SLICES-CONSTRUCT-54D1E67F
title: "StreamScanner token slices construct str without UTF-8 validation"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-19
updated: 2026-05-19
target: "stdlib/std/streamio/scanner/state.nepl; nodesrc/test_stdlib_streamio_scanner_boundary.js; tests/stdlib/streamio.n.md"
---

# ISS-20260519T064344008Z-STREAMSCANNER-TOKEN-SLICES-CONSTRUCT-54D1E67F: StreamScanner token slices construct str without UTF-8 validation

## 概要

`StreamScanner` は `ReadStream::Bytes` や file input 由来の任意 byte 列を scan できるが、`stream_scanner_slice_to_str_result` は byte range だけを検査して `string_from_mem_unchecked_result` で `str` を構築していた。invalid UTF-8 が compiler / stdlib の検証なしに `str` として境界を越えられる。

## 対象

- `stdlib/std/streamio/scanner/state.nepl; nodesrc/test_stdlib_streamio_scanner_boundary.js; tests/stdlib/streamio.n.md`

## 根拠

- `stdlib/std/streamio/scanner/state.nepl` の `stream_scanner_slice_to_str_result` は `io_bytebuf_ptr_ref` から得た `MemPtr<u8>` と `start` / `tlen` で pointer/length pair を作り、旧実装では `string_from_mem_unchecked_result` に渡していた。
- `StreamScanner` の入力は `ReadStream::Text` に限られず、`ReadStream::Bytes` や file input からも作られるため、token bytes が UTF-8 である保証は scanner 境界で証明する必要がある。

## 問題

`StreamScanner` は外部 byte input を扱う public API であるにもかかわらず、token slice の `str` 化を unchecked constructor に委譲していた。これは `ByteBuf` extent の検査と `str` の UTF-8 invariant を別物として扱えておらず、Stage 6 の raw-memory-backed API 境界として不十分だった。

## 影響

invalid UTF-8 を `str` として公開できるため、型安全性と text API の前提が崩れる。特に self-host compiler の lexer/parser が scanner を使う場合、入力 source が妥当な `str` であるという前提が実装規約だけに落ちてしまう。

## 修正方針

- `stream_scanner_slice_to_str_result` は従来の range check を維持した上で、`string_from_utf8_mem_result` に委譲して UTF-8 検証後だけ `str` を構築する。
- scanner state boundary の source policy で、checked constructor の利用と unchecked constructor の再導入禁止を固定する。
- `ReadStream::Bytes` から invalid UTF-8 token を読む doctest を追加し、runtime 上も invalid token が `str` として露出しないことを確認する。

## 検証

- `node nodesrc/test_stdlib_streamio_scanner_boundary.js`: pass
- `node nodesrc/test_nmd_report_metadata_policy.js`: pass
- `node nodesrc/tests.js -i tests/stdlib/streamio.n.md --no-tree -o tmp/agent1-streamio-scanner-utf8-token-boundary.json -j 1 --dist web/dist --assert-io`: total=16, passed=16
- `node nodesrc/run_source_policy_regressions.js --warn-only`: exit 0

## 関連

- parent: `ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84`
- Stage: `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6
