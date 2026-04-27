---
id: ISS-20260427T070602957Z-JSON-STDLIB-DOC-COMMENTS-STILL-CONTA-19D460FE
title: "json stdlib doc comments still contain generated boilerplate"
area: stdlib
status: verified
resolved: true
priority: P2
type: doc
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/encoding/json.nepl; nodesrc/test_stdlib_json_doc_no_boilerplate.js"
source: ISS-20260426T060358681Z-STDLIB-DOC-COMMENTS-STILL-CONTAIN-GE-2D7384D1
---

# ISS-20260427T070602957Z-JSON-STDLIB-DOC-COMMENTS-STILL-CONTA-19D460FE: json stdlib doc comments still contain generated boilerplate

## 概要

stdlib/alloc/encoding/json.nepl still contains generated doc comment boilerplate such as 主な用途, 定義済み処理, 薄いラッパ, and generic move/rebind notes. These comments do not document ownership transfer, array/object construction, or JsonValue accessor semantics.

## 対象

- `stdlib/alloc/encoding/json.nepl; nodesrc/test_stdlib_json_doc_no_boilerplate.js`

## 根拠

- `stdlib/alloc/encoding/json.nepl` の constructor / accessor 周辺に生成テンプレート由来の boilerplate comment が残っていた。
- `rg` で `主な用途` / `定義済み処理` / `薄いラッパ` / move-rebind 注意の残存を確認した。

## 問題

stdlib/alloc/encoding/json.nepl still contains generated doc comment boilerplate such as 主な用途, 定義済み処理, 薄いラッパ, and generic move/rebind notes. These comments do not document ownership transfer, array/object construction, or JsonValue accessor semantics.

## 影響

The JSON stdlib API is useful for self-hosting diagnostics and interchange, but generated placeholders make constructor and accessor contracts harder to review and allow the doc debt to regress unnoticed.

## 修正方針

Replace the generated blocks with concrete Japanese nm-style comments for JSON constructors and accessors. Add a source policy regression that fails if the boilerplate phrases return to stdlib/alloc/encoding/json.nepl.

## 修正内容

- `JsonValue`、JSON constructor、`json_is_null`、`json_as_bool` / `json_as_number` / `json_as_string` のコメントを実装契約に合わせた nm コメントへ置き換えた。
- `nodesrc/test_stdlib_json_doc_no_boilerplate.js` を追加し、生成テンプレート文言の再混入と JSON API 仕様説明の欠落を検出するようにした。
- CI と `doc/testing.md` の source policy regressions に同テストを追加した。

## 検証

- `node nodesrc/test_stdlib_json_doc_no_boilerplate.js`: pass
- `node nodesrc/tests.js -i stdlib/alloc/encoding/json.nepl --no-tree -o tmp/json-doc-boilerplate.json -j 1`: `total=1`, `passed=1`
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-json-doc-boilerplate.json -j 4`: `total=418`, `passed=418`
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass（CRLF 変換 warning のみ）
