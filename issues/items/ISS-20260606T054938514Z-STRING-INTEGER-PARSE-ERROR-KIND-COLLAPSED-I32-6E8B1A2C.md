---
id: ISS-20260606T054938514Z-STRING-INTEGER-PARSE-ERROR-KIND-COLLAPSED-I32-6E8B1A2C
title: "string integer parser collapses parse errors into i32"
area: stdlib
status: open
resolved: false
priority: P2
type: architecture
created: 2026-06-06
updated: 2026-06-06
target: "stdlib/alloc/string/integer/parse.nepl"
---

# ISS-20260606T054938514Z-STRING-INTEGER-PARSE-ERROR-KIND-COLLAPSED-I32-6E8B1A2C: string integer parser collapses parse errors into i32

## 概要

`stdlib/alloc/string/integer/parse.nepl` の public parse API は、invalid radix、無効 digit、空入力、`-` だけの入力、prefix 付き入力、overflow、range 外 byte access をいずれも `Result::Err 1` へ畳んでいる。

2026-06-06 の Agent2 IntegerParse doc slice では、この挙動を現状互換境界として doc comment と source policy に明記した。ただし、Zenn 方針の「エラーは enum で管理し、表示と分離し、`match` による網羅性検査が効く形にする」に対しては未達であるため、この issue で typed enum parse error への整理を追跡する。

## 問題

`Result::Err 1` だけでは、呼び出し側が次の条件を静的に区別できない。

- invalid radix
- invalid digit
- empty input
- sign without digit
- unsupported prefix form
- positive overflow
- negative overflow
- input range / byte access boundary failure

このため、利用者は error kind を `match` で網羅的に扱えず、doc comment とテストも `1` の意味を現状説明として補足する必要がある。

## 修正方針

`StringIntegerParseError` のような enum を導入し、parse API の失敗 payload を typed error へ移す。

互換 API が必要な場合は、typed error から legacy `i32` へ落とす adapter を別に置く。error 表示は std 側または dedicated display helper に分離し、parse core は enum data だけを返す。

## 完了条件

- invalid radix、invalid digit、empty input、sign without digit、unsupported prefix、overflow、range boundary を enum variant として表せる。
- public parse API が typed enum error を返す、または typed API と legacy adapter の境界が文書化される。
- `match` による網羅性検査を使う doctest / normal test が追加される。
- `nodesrc/test_alloc_string_doc_report_contract.js` または dedicated source policy が、`Result::Err 1` の新規固定化を拒否する。

## 検証

- `node nodesrc/test_alloc_string_doc_report_contract.js`
- `node nodesrc/test_stdlib_string_integer_boundary.js`
- `node nodesrc/test_stdlib_documentation_contract.js`
- focused `nodesrc/tests.js` for `stdlib/alloc/string/integer/parse.nepl`
