---
id: ISS-20260606T073427291Z-STRING-SLICE-CHAR-ERROR-KIND-COLLAPSED-STR-4F9E2A81
title: "string slice and char decode errors are collapsed into str payloads"
area: stdlib
status: open
resolved: false
priority: P2
type: architecture
created: 2026-06-06
updated: 2026-06-06
target: "stdlib/alloc/string/slice"
---

# ISS-20260606T073427291Z-STRING-SLICE-CHAR-ERROR-KIND-COLLAPSED-STR-4F9E2A81: string slice and char decode errors are collapsed into str payloads

## 概要

`stdlib/alloc/string/slice/byte.nepl` と `stdlib/alloc/string/slice/char.nepl` の Result API は、UTF-8 boundary 違反、char index 範囲外、invalid scalar、invalid leading byte、invalid continuation、truncated UTF-8、invalid char slice range を `Result::Err str` の文字列 payload として返している。

2026-06-06 の Agent2 StringSlice doc slice では、この挙動を現状互換境界として doc comment と source policy に明記した。ただし、Zenn 方針の「エラーは enum で管理し、表示と分離し、`match` による網羅性検査が効く形にする」に対しては未達であるため、この issue で typed enum error への整理を追跡する。

## 問題

`Result::Err str` だけでは、呼び出し側が次の条件を静的に区別できない。

- byte slice の UTF-8 boundary 違反
- byte index の out-of-bounds
- char index の out-of-bounds
- Unicode scalar value として不正な code point
- invalid leading byte
- invalid continuation byte
- truncated UTF-8 sequence
- char slice range の逆転や存在しない char index

このため、利用者は error kind を `match` で網羅的に扱えず、診断表示用の文字列と error data が分離されていない。

## 修正方針

`StringSliceError` または `StringCharDecodeError` のような enum を導入し、slice / char Result API の失敗 payload を typed error へ移す。

互換 API が必要な場合は、typed error から legacy `str` payload へ落とす adapter を別に置く。error 表示は std 側または dedicated display helper に分離し、slice / char core は enum data だけを返す。

## 完了条件

- UTF-8 boundary、byte out-of-bounds、char out-of-bounds、invalid scalar、invalid lead、invalid continuation、truncated sequence、invalid char slice range を enum variant として表せる。
- public Result API が typed enum error を返す、または typed API と legacy adapter の境界が文書化される。
- `match` による網羅性検査を使う doctest / normal test が追加される。
- `nodesrc/test_alloc_string_doc_report_contract.js` または dedicated source policy が、`Result::Err str` の新規固定化を互換境界としてしか許さない。

## 検証

- `node nodesrc/test_alloc_string_doc_report_contract.js`
- `node nodesrc/test_stdlib_string_slice_boundary.js`
- `node nodesrc/test_stdlib_documentation_contract.js`
- focused `nodesrc/tests.js` for `stdlib/alloc/string/slice/byte.nepl` and `stdlib/alloc/string/slice/char.nepl`
