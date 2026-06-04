---
id: ISS-20260604T034125917Z-CHAR-UTF-8-BYTE-ACCESSORS-RELY-ON-CA-AC31C3D4
title: "char UTF-8 byte accessors rely on caller length checks instead of typed absence"
area: stdlib
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-06-04
updated: 2026-06-04
target: stdlib/core/char.nepl
---

# ISS-20260604T034125917Z-CHAR-UTF-8-BYTE-ACCESSORS-RELY-ON-CA-AC31C3D4: char UTF-8 byte accessors rely on caller length checks instead of typed absence

## 概要

Subagent audit found char_utf8_byte1/2/3 requiring callers to know byte length before access, rather than returning Option/Result for absent bytes. This conflicts with Zenn guidance to express nullable/absent states through Option and keep invalid state out of ordinary values.

## 対象

- `stdlib/core/char.nepl`

## 根拠

- 未記入

## 問題

Subagent audit found char_utf8_byte1/2/3 requiring callers to know byte length before access, rather than returning Option/Result for absent bytes. This conflicts with Zenn guidance to express nullable/absent states through Option and keep invalid state out of ordinary values.

## 影響

Callers can accidentally read byte positions that do not exist for ASCII or shorter UTF-8 encodings, and the static checker cannot force match coverage for absent bytes.

## 修正方針

Introduce char_utf8_byte_at returning Option i32 or an encoded UTF-8 struct with explicit length, and make raw byteN helpers private/internal or documented precondition-only helpers.

## 検証

Add doctests and regular tests for ASCII, 2-byte, 3-byte, invalid index, and boundary access.

## 修正結果

- `core/char` に `char_utf8_byte_at c idx -> Option i32` を追加し、存在しない UTF-8 byte index を `None` として返す contract にした。
- `char_utf8_byte1` / `char_utf8_byte2` / `char_utf8_byte3` は public API から外し、`char_utf8_byte_at` の内部 helper に閉じた。
- `alloc/io/bytebuilder/append` は raw tail byte helper を直接呼ばず、`char_utf8_byte_at` を `match Option::Some` / `Option::None` で扱う private helper 経由にした。
- `byte_builder_push_utf8_tail` は `char_utf8_len` 由来の内部前提を受ける helper なので private 化した。
- `doc/neplg2/char_stdlib_integration_plan.md` の旧 `char_utf8_byte0..3` public contract を、`char_utf8_byte0` と `char_utf8_byte_at -> Option i32` に更新した。
- source policy に `char_utf8_byte1/2/3` の public 化禁止、`char_utf8_byte_at` の `Option i32` contract、bytebuilder 側の `match` 経由を追加した。

## 実行した検証

- `node nodesrc/test_stdlib_char_utf8_byte_contract.js`
- `node nodesrc/test_core_char_doc_report_contract.js`
- `node nodesrc/test_stdlib_documentation_contract.js`
- `node nodesrc/tests.js -i stdlib/core/char.nepl -i stdlib/alloc/io/bytebuilder/append.nepl -i stdlib/alloc/io/bytebuilder/build.nepl -i tests/stdlib/char_utf8_byte_at.n.md -i tests/stdlib/string_char.n.md --no-tree -o tmp/agent2-char-utf8-byte-option-focused-3.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/run_source_policy_regressions.js --warn-only`

`run_source_policy_regressions.js --warn-only` では、今回追加した char / bytebuilder / documentation contract は通過した。既存の compiler / resource / diagnostic 系 warning は 5 件残っている。
