---
id: ISS-20260604T033842259Z-STREAMIO-SCANNER-AND-WRITER-APIS-STI-EFC1568E
title: "streamio scanner and writer APIs still mix string errors sentinel fallbacks and non-Result effects"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-06-04
updated: 2026-06-05
target: "stdlib/std/streamio/scanner, stdlib/std/streamio/writer.nepl"
---

# ISS-20260604T033842259Z-STREAMIO-SCANNER-AND-WRITER-APIS-STI-EFC1568E: streamio scanner and writer APIs still mix string errors sentinel fallbacks and non-Result effects

## 概要

Subagent audit found StreamScanner helpers returning Result ... str while compatibility facades collapse cursor corruption into 0 or unit. Stream writer write/flush/close paths expose side effects without returning typed failures. This conflicts with Zenn guidance to use Option/Result and enum, avoid silent no-op, and keep side effects explicit at the surface.

## 対象

- `stdlib/std/streamio/scanner, stdlib/std/streamio/writer.nepl`

## 根拠

- Zenn 記事の「成功や失敗は Option / Result を用いて明示的に扱う」「error は enum data として管理し表示と分離する」「silent no-op を避ける」「match による網羅性検査を活用する」という方針。
- `std/streamio` は stdin/stdout/stderr/file/text/binary stream という platform-facing boundary に近いため、失敗を sentinel や raw string に潰すと caller が静的に扱えない。

## 問題

Subagent audit found StreamScanner helpers returning Result ... str while compatibility facades collapse cursor corruption into 0 or unit. Stream writer write/flush/close paths expose side effects without returning typed failures. This conflicts with Zenn guidance to use Option/Result and enum, avoid silent no-op, and keep side effects explicit at the surface.

## 影響

Scanner cursor corruption, malformed token, EOF, invalid UTF-8, append allocation failure, stdout/stderr write failure, and close failure cannot be handled by exhaustive match, so streamio users must infer failures from sentinel values or lost effects.

## 修正方針

Define StreamScannerError and StreamWriterError enums, make *_result APIs the primary public surface, downgrade legacy sentinel wrappers to documented compatibility paths, and add regular tests for cursor corruption, malformed input, EOF, invalid UTF-8, allocation failure, write failure, flush failure, and close failure.

## 検証

- `StreamScannerError` と `StreamWriterErrorKind` / `StreamWriterError` を追加し、scanner / writer の primary API を `Result` にした。
- 既存互換 API は残したが、typed `*_result` へ委譲し、失敗を丸めることを doc comment に明記した。
- scanner parser は token byte access を `stream_scanner_byte_at_result` へ統一し、EOF / malformed token / invalid UTF-8 / cursor failure を enum error として返す。
- writer は append / flush / close の失敗時に recover 可能な writer owner を `StreamWriterError` payload に戻す。
- `u32` / `u64` の primitive `Clone` / `Copy` 実装を追加し、streamio の unsigned conversion が resource leak にならないようにした。
- 検証済み: `node nodesrc/test_stdlib_streamio_scanner_boundary.js`、`node nodesrc/test_stdlib_streamio_writer_boundary.js`、`node nodesrc/test_stdlib_streamio_no_unsafe_unwraps.js`、`node nodesrc/tests.js -i tests/stdlib/streamio.n.md -o tmp/streamio-tests.json --assert-io -j 1`、`node nodesrc/test_stdlib_documentation_contract.js`、`node nodesrc/run_source_policy_regressions.js --warn-only`、`node nodesrc/issues.js check --dir issues`、`git diff --check`、`trunk build`、`node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/agent2-streamio-typed-error-playground-editor.json`。
- `node nodesrc/run_source_policy_regressions.js --warn-only` は今回対象の streamio / documentation policy が通過し、既存の `test_resource_gate_order.js` と `test_diagnostic_code_first_boundary.js` だけを warning として報告した。
