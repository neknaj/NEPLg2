---
id: ISS-20260606T052400000Z-STRING-FLOAT-PARSE-ERROR-KIND-COLLAPSED-I32-4B21D9A7
title: "string float parser collapses parse errors into i32"
area: stdlib
status: open
resolved: false
priority: P2
type: architecture
created: 2026-06-06
updated: 2026-06-06
target: "stdlib/alloc/string/float/parse.nepl"
---

# ISS-20260606T052400000Z-STRING-FLOAT-PARSE-ERROR-KIND-COLLAPSED-I32-4B21D9A7: string float parser collapses parse errors into i32

## 概要

`stdlib/alloc/string/float/parse.nepl` の `to_f64` / `to_f32` は parse error を `Result::Err 1` に畳んでいる。Zenn 方針では、失敗の種類は数値や文字列ではなく enum で管理し、`match` による網羅性検査が効く形にする必要がある。

## 対象

- `stdlib/alloc/string/float/parse.nepl`
- `to_f64`
- `to_f32`
- `float_parse_byte_or_invalid`

## 問題

現状の parser は empty input、sign only、dot only、missing exponent、trailing byte、unsupported symbolic value などをすべて同じ `i32` error payload に畳む。呼び出し側は失敗理由を静的に分岐できず、将来の diagnostics や serializer / deserializer の error 表示で文字列比較や数値 sentinel に依存しやすくなる。

今回の documentation slice では互換境界として現状を明記するが、typed enum error への移行は未実装である。

## 修正方針

- `FloatParseErrorKind` のような enum を導入し、empty input、missing digit、missing exponent digit、trailing byte、unsupported symbolic value などを区別する。
- `to_f64_result` 相当の typed error API を用意し、互換 API が必要なら `Result f64 i32` への変換を薄い互換層へ閉じる。
- `to_f32` は `to_f64` の typed error をそのまま伝播し、error reason を潰さない。
- error 自体と display / user-facing message を分離する。

## 検証

- `node nodesrc/test_alloc_string_doc_report_contract.js`
- `node nodesrc/test_stdlib_string_float_boundary.js`
- `node nodesrc/tests.js -i stdlib/alloc/string/float/parse.nepl --no-tree -o tmp/agent2-string-float-parse-doc-slice.json -j 1 --dist web/dist --assert-io`

