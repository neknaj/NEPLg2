---
id: ISS-20260605T203044028Z-STRING-INTEGER-FORMAT-ERROR-KIND-COLLAPSED-I32-8C63F92A
title: "string integer formatter collapses format errors into i32"
area: stdlib
status: open
resolved: false
priority: P2
type: architecture
created: 2026-06-06
updated: 2026-06-06
target: "stdlib/alloc/string/integer/format.nepl"
---

# ISS-20260605T203044028Z-STRING-INTEGER-FORMAT-ERROR-KIND-COLLAPSED-I32-8C63F92A: string integer formatter collapses format errors into i32

## 概要

`stdlib/alloc/string/integer/format.nepl` の `from_i32_radix` / `from_i64_radix` / `from_u128_radix` / `from_i128_radix` は、失敗理由を `Result::Err i32` に畳んでいる。Zenn 方針では、失敗の種類は数値や文字列ではなく enum で管理し、`match` による網羅性検査が効く形にする必要がある。

## 対象

- `stdlib/alloc/string/integer/format.nepl`
- `from_i32_radix`
- `from_i64_radix`
- `from_u128_radix`
- `from_i128_radix`

## 問題

現状の formatter は invalid radix を `Result::Err 1`、allocation / builder failure を `Result::Err 12` として返す。呼び出し側は数値 payload を知っていないと失敗理由を分岐できず、将来の diagnostics や error display で数値 sentinel に依存しやすくなる。

今回の documentation slice では互換境界として現状を明記するが、typed enum error への移行は未実装である。`from_i32` / `from_i64` / `from_u128` / `from_i128` は失敗時に `"0"` fallback を返す互換 API なので、typed error API の設計時には Result API と fallback API の境界を保つ必要がある。

## 修正方針

- `IntegerFormatErrorKind` のような enum を導入し、invalid radix、allocation failure、builder failure を区別する。
- `from_*_radix` は typed error を返す API へ移し、互換 API が必要なら `Result str i32` への変換を薄い互換層へ閉じる。
- `from_i32` / `from_i64` / `from_u128` / `from_i128` の `"0"` fallback は互換 API として明示し、error reason が必要な新規コードは Result API を使う契約にする。
- error 自体と display / user-facing message を分離する。

## 検証

- `node nodesrc/test_alloc_string_doc_report_contract.js`
- `node nodesrc/test_stdlib_string_integer_boundary.js`
- `node nodesrc/test_stdlib_string_no_unsafe_unwraps.js`
- `node nodesrc/tests.js -i stdlib/alloc/string/integer/format.nepl --no-tree -o tmp/agent2-string-integer-format-doc-slice.json -j 1 --dist web/dist --assert-io`
