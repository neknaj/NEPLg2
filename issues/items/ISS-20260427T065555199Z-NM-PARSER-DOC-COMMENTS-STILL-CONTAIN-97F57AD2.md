---
id: ISS-20260427T065555199Z-NM-PARSER-DOC-COMMENTS-STILL-CONTAIN-97F57AD2
title: "nm parser doc comments still contain generated boilerplate"
area: stdlib
status: verified
resolved: true
priority: P2
type: doc
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/nm/parser.nepl; nodesrc/test_stdlib_nm_parser_doc_no_boilerplate.js"
source: ISS-20260426T060358681Z-STDLIB-DOC-COMMENTS-STILL-CONTAIN-GE-2D7384D1
---

# ISS-20260427T065555199Z-NM-PARSER-DOC-COMMENTS-STILL-CONTAIN-97F57AD2: nm parser doc comments still contain generated boilerplate

## 概要

stdlib/nm/parser.nepl still contains generated doc comment boilerplate such as 主な用途, 定義済み処理, 薄いラッパ, and generic move/rebind notes. These comments hide the actual extended-markdown parsing rules and JSON emission contracts.

## 対象

- `stdlib/nm/parser.nepl; nodesrc/test_stdlib_nm_parser_doc_no_boilerplate.js`

## 根拠

- `stdlib/nm/parser.nepl` の parser state struct、block predicate、section close helper、block parser、JSON emitter 周辺に生成テンプレート由来の boilerplate comment が残っていた。
- `rg` で `主な用途` / `定義済み処理` / `薄いラッパ` / move-rebind 注意の残存を確認した。

## 問題

stdlib/nm/parser.nepl still contains generated doc comment boilerplate such as 主な用途, 定義済み処理, 薄いラッパ, and generic move/rebind notes. These comments hide the actual extended-markdown parsing rules and JSON emission contracts.

## 影響

The nm parser is a self-hosting-facing library, but its docs do not explain block parsing, section closing, inline escaping, or JSON emission enough for review and maintenance. Generated placeholders can also regress unnoticed.

## 修正方針

Replace the generated blocks in nm parser with concrete Japanese nm-style comments for parser state structs, block predicates, section close helpers, block parsers, and JSON emitters. Add a source policy regression that fails if the boilerplate phrases return to stdlib/nm/parser.nepl.

## 修正内容

- `FenceRes` / `ParaRes`、block predicate、section close helper、`parse_heading` / `parse_fence` / `parse_paragraph`、JSON emitter のコメントを実装契約に合わせた nm コメントへ置き換えた。
- `nodesrc/test_stdlib_nm_parser_doc_no_boilerplate.js` を追加し、生成テンプレート文言の再混入と parser 仕様説明の欠落を検出するようにした。
- CI と `doc/testing.md` の source policy regressions に同テストを追加した。

## 検証

- `node nodesrc/test_stdlib_nm_parser_doc_no_boilerplate.js`: pass
- `node nodesrc/tests.js -i stdlib/nm/parser.nepl --no-tree -o tmp/nm-parser-doc-boilerplate.json -j 1`: `total=3`, `passed=3`
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-nm-parser-doc-boilerplate.json -j 4`: `total=418`, `passed=418`
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass（CRLF 変換 warning のみ）
