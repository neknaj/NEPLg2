---
id: ISS-20260604T033842259Z-STREAMIO-SCANNER-AND-WRITER-APIS-STI-EFC1568E
title: "streamio scanner and writer APIs still mix string errors sentinel fallbacks and non-Result effects"
area: stdlib
status: open
resolved: false
priority: P1
type: architecture
created: 2026-06-04
updated: 2026-06-04
target: "stdlib/std/streamio/scanner, stdlib/std/streamio/writer.nepl"
---

# ISS-20260604T033842259Z-STREAMIO-SCANNER-AND-WRITER-APIS-STI-EFC1568E: streamio scanner and writer APIs still mix string errors sentinel fallbacks and non-Result effects

## 概要

Subagent audit found StreamScanner helpers returning Result ... str while compatibility facades collapse cursor corruption into 0 or unit. Stream writer write/flush/close paths expose side effects without returning typed failures. This conflicts with Zenn guidance to use Option/Result and enum, avoid silent no-op, and keep side effects explicit at the surface.

## 対象

- `stdlib/std/streamio/scanner, stdlib/std/streamio/writer.nepl`

## 根拠

- 未記入

## 問題

Subagent audit found StreamScanner helpers returning Result ... str while compatibility facades collapse cursor corruption into 0 or unit. Stream writer write/flush/close paths expose side effects without returning typed failures. This conflicts with Zenn guidance to use Option/Result and enum, avoid silent no-op, and keep side effects explicit at the surface.

## 影響

Scanner cursor corruption, malformed token, EOF, invalid UTF-8, append allocation failure, stdout/stderr write failure, and close failure cannot be handled by exhaustive match, so streamio users must infer failures from sentinel values or lost effects.

## 修正方針

Define StreamScannerError and StreamWriterError enums, make *_result APIs the primary public surface, downgrade legacy sentinel wrappers to documented compatibility paths, and add regular tests for cursor corruption, malformed input, EOF, invalid UTF-8, allocation failure, write failure, flush failure, and close failure.

## 検証

Run streamio focused doctests, source policy regressions, and future cfg-test-style scanner/writer error matrix tests.
