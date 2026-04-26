---
id: ISS-20260426T060250100Z-JSONVALUE-STORES-STRUCTURED-JSON-PAY-8494C374
title: "JsonValue stores structured JSON payloads as raw i32 handles"
area: stdlib
status: verified
resolved: true
priority: P2
type: architecture
created: 2026-04-26
updated: 2026-04-26
target: stdlib/alloc/encoding/json.nepl
---

# ISS-20260426T060250100Z-JSONVALUE-STORES-STRUCTURED-JSON-PAY-8494C374: JsonValue stores structured JSON payloads as raw i32 handles

## 概要

JsonValue::String、JsonValue::Array、JsonValue::Object が raw i32 payload を保持し、Object の実体も未定義とコメントされている。文字列、配列、object の所有権と型が stdlib API から見えず、parser/serializer/builder として安全に使えない。

## 対象

- `stdlib/alloc/encoding/json.nepl`

## 根拠

- 未記入

## 問題

JsonValue::String、JsonValue::Array、JsonValue::Object が raw i32 payload を保持し、Object の実体も未定義とコメントされている。文字列、配列、object の所有権と型が stdlib API から見えず、parser/serializer/builder として安全に使えない。

## 影響

diagnostic JSON や self-host artifact metadata を組み立てるとき、無効 pointer や layout mismatch を型で防げない。JSON module 名に対して実際には typed JSON value builder になっていない。

## 修正方針

String は str、Array は Vec<JsonValue>、Object は明示的な JsonMember/Vec<JsonMember> など typed representation へ移行し、serialize/escape/build API と ownership 契約を定義する。必要に応じて builder を分ける。
ただし `str` と raw `i32` handle の型分離は core 側の問題であり、`json_string 0` の compile-time rejection は `ISS-20260426T074114888Z-STR-UNIFIES-WITH-I32-AND-ACCEPTS-RAW-A824A1D7` で扱う。

## 検証

string/array/object の roundtrip、escape、nested value、array/object の invalid raw handle を排除する compile_fail を追加する。
string payload の raw i32 rejection は core の str/i32 分離 issue の回帰テストで固定する。

## 対応

`JsonValue::String` を `str` payload、`JsonValue::Array` を `Vec<JsonValue>` payload、`JsonValue::Object` を `Vec<JsonMember>` payload へ移行した。
array/object builder と serializer を追加し、`json_escape` は byte を `JsonEscapeKind` enum へ分類してから `match` で escape 文字列を選ぶようにした。
既存 `stdlib/tests/json.n.md` は typed builder API に同期し、`tests/stdlib/json_typed_values.n.md` で string/array/object roundtrip、nested serialize、escape、array/object raw handle compile_fail を固定した。

## 検証結果

- `node nodesrc/tests.js -i tests/stdlib/json_typed_values.n.md --no-tree -o tmp/json-typed-values-focused.json -j 1`: `total=6`, `passed=6`, `failed=0`
- `node nodesrc/tests.js -i stdlib/alloc/encoding/json.nepl -i stdlib/tests/json.n.md -i tests/stdlib/json_typed_values.n.md --no-tree -o tmp/json-typed-values-suite.json -j 1`: `total=8`, `passed=8`, `failed=0`
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/json-typed-values-stdlib-full.json -j 4`: `total=404`, `passed=404`, `failed=0`
