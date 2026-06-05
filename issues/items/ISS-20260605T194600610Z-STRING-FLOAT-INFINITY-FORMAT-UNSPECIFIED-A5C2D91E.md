---
id: ISS-20260605T194600610Z-STRING-FLOAT-INFINITY-FORMAT-UNSPECIFIED-A5C2D91E
title: "string float formatter does not define Infinity behavior"
area: stdlib
status: open
resolved: false
priority: P2
type: doc
created: 2026-06-06
updated: 2026-06-06
target: "stdlib/alloc/string/float/format.nepl"
---

# ISS-20260605T194600610Z-STRING-FLOAT-INFINITY-FORMAT-UNSPECIFIED-A5C2D91E: string float formatter does not define Infinity behavior

## 概要

`stdlib/alloc/string/float/format.nepl` の `from_f64_result` は `NaN` を `"nan"` として扱うが、positive infinity / negative infinity の public contract をまだ定義していない。Zenn 記事の方針では、戻り値や境界条件の分岐条件を doc comment に明記し、型情報だけでは分からないことを契約と現状に分けて記載する必要がある。

## 対象

- `stdlib/alloc/string/float/format.nepl`
- `from_f64_result`
- `from_f64`
- `from_f32`

## 問題

現状の formatter は固定小数 formatting を前提にしており、指数表記も Infinity 表記も contract として定義していない。`NaN` だけは `"nan"` を返す分岐があるため、Infinity が未規定のままだと floating special value の扱いが不均衡になる。

この issue は今回の documentation slice では仕様化しない。`from_f64_result` の doc comment では Infinity を未規定と明記し、この issue で後続の設計判断を追跡する。

## 修正方針

- `f64` special value の public contract を定義する。
- `NaN`、positive infinity、negative infinity、finite value を `match` 相当の明示的な分岐として扱える形に整理する。
- 可能なら string error payload / i32 error payload ではなく、typed enum error へ寄せる。
- `from_f64` / `from_f32` の fallback API と `Result` API の責務差を維持する。

## 検証

- `node nodesrc/test_alloc_string_doc_report_contract.js`
- `node nodesrc/test_stdlib_string_float_boundary.js`
- `node nodesrc/tests.js -i stdlib/alloc/string/float/format.nepl --no-tree -o tmp/agent2-string-float-format-doc-slice.json -j 1 --dist web/dist --assert-io`
