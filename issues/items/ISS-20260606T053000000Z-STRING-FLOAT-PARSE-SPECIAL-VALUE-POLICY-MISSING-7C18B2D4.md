---
id: ISS-20260606T053000000Z-STRING-FLOAT-PARSE-SPECIAL-VALUE-POLICY-MISSING-7C18B2D4
title: "string float parse and format special value policy is incomplete"
area: stdlib
status: open
resolved: false
priority: P2
type: doc
created: 2026-06-06
updated: 2026-06-06
target: "stdlib/alloc/string/float"
---

# ISS-20260606T053000000Z-STRING-FLOAT-PARSE-SPECIAL-VALUE-POLICY-MISSING-7C18B2D4: string float parse and format special value policy is incomplete

## 概要

`stdlib/alloc/string/float/format.nepl` は `NaN` を `"nan"` として formatting する一方、`stdlib/alloc/string/float/parse.nepl` は `nan` / `inf` などの記号名を parse しない。この非対称性が public contract として妥当なのか、round-trip を提供するべきなのかが標準 library 全体の方針として未整理である。

## 対象

- `stdlib/alloc/string/float/format.nepl`
- `stdlib/alloc/string/float/parse.nepl`
- `from_f64_result`
- `to_f64`
- `to_f32`

## 問題

現在は format 側の `NaN -> "nan"` と parse 側の symbolic value 拒否を個別に document している。しかし、`from_f64` の出力を `to_f64` が受理しない場合があることを、round-trip 非保証として標準契約に明示するか、特殊値 parse を追加するかの判断が残っている。

`ISS-20260605T194600610Z-STRING-FLOAT-INFINITY-FORMAT-UNSPECIFIED-A5C2D91E` は format 側の Infinity contract を扱う。この issue は parse / format をまたぐ special value policy と round-trip contract を扱う。

## 修正方針

- finite value、NaN、positive infinity、negative infinity について、format と parse の public contract を表として整理する。
- round-trip を保証する範囲と保証しない範囲を明記する。
- 特殊値 parse を追加する場合は、typed enum error と `match` による分岐を前提にする。
- 追加しない場合は、`from_f64` の output が常に `to_f64` の input として valid ではないことを facade doc に明記する。

## 検証

- `node nodesrc/test_alloc_string_doc_report_contract.js`
- `node nodesrc/test_stdlib_string_float_boundary.js`
- `node nodesrc/tests.js -i stdlib/alloc/string/float/parse.nepl -i stdlib/alloc/string/float/format.nepl --no-tree -o tmp/agent2-string-float-special-policy.json -j 1 --dist web/dist --assert-io`

