---
id: ISS-20260429T053406066Z-NM-PARSER-DOC-POLICY-STILL-REQUIRES--F7604AA4
title: "nm parser doc policy still requires removed local line scanner helper"
area: nodesrc
status: fixed
resolved: true
priority: P1
type: test
created: 2026-04-29
updated: 2026-04-29
target: "nodesrc/test_stdlib_nm_parser_doc_no_boilerplate.js, stdlib/nm/parser.nepl"
---

# ISS-20260429T053406066Z-NM-PARSER-DOC-POLICY-STILL-REQUIRES--F7604AA4: nm parser doc policy still requires removed local line scanner helper

## 概要

GitHub Actions run 25092323380 fails Source policy regressions because nodesrc/test_stdlib_nm_parser_doc_no_boilerplate.js still requires the removed nm_line_end documentation phrase after line scanning moved to alloc/string/scanner.

## 対象

- `nodesrc/test_stdlib_nm_parser_doc_no_boilerplate.js, stdlib/nm/parser.nepl`

## 根拠

- GitHub Actions run `25092323380` の Source policy regressions は `nodesrc/test_stdlib_nm_parser_doc_no_boilerplate.js` の `stdlib/nm/parser.nepl must document line scanner contract` で停止した。
- `ISS-20260429T024625042Z-STDLIB-LACKS-REUSABLE-BYTE-SCANNER-A-3453D5E0` の対応で、NM parser の local `nm_line_end` / `nm_next_line_pos` は削除され、`alloc/string/scanner` の `str_line_end` / `str_next_line_pos` へ責務が移っている。
- 旧 source policy は削除済み helper 名 `nm_line_end` の doc phrase を要求しており、現在の責務分割を検査していなかった。

## 問題

GitHub Actions run 25092323380 fails Source policy regressions because nodesrc/test_stdlib_nm_parser_doc_no_boilerplate.js still requires the removed nm_line_end documentation phrase after line scanning moved to alloc/string/scanner.

## 影響

main CI stops before the main doctest jobs, and the source policy no longer checks the current nm/parser responsibility boundary.

## 修正方針

Update the nm/parser documentation and source policy to assert the current nm_read_line wrapper and scanner::str_line_end delegation contract instead of the deleted nm_line_end helper.

## 検証

Run node nodesrc/test_stdlib_nm_parser_doc_no_boilerplate.js and node nodesrc/test_stdlib_byte_scanner_helpers_boundary.js.

## 対応結果

`stdlib/nm/parser.nepl` の `nm_read_line` コメントに、行末探索は `scanner::str_line_end` へ委譲し、NM parser 側は CRLF 正規化だけを担当することを明記した。

`nodesrc/test_stdlib_nm_parser_doc_no_boilerplate.js` は、削除済み `nm_line_end` の文言ではなく、現在の `nm_read_line` wrapper と `scanner::str_line_end` delegation contract を要求するように更新した。

検証:

- `node nodesrc/test_stdlib_nm_parser_doc_no_boilerplate.js`
- `node nodesrc/test_stdlib_byte_scanner_helpers_boundary.js`
- `node nodesrc/issues.js index`
- `node nodesrc/issues.js check`
