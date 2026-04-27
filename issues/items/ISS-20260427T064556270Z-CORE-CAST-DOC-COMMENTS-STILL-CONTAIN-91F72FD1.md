---
id: ISS-20260427T064556270Z-CORE-CAST-DOC-COMMENTS-STILL-CONTAIN-91F72FD1
title: "core/cast doc comments still contain generated boilerplate"
area: stdlib
status: verified
resolved: true
priority: P2
type: doc
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/core/cast.nepl; nodesrc/test_stdlib_cast_doc_no_boilerplate.js"
source: ISS-20260426T060358681Z-STDLIB-DOC-COMMENTS-STILL-CONTAIN-GE-2D7384D1
---

# ISS-20260427T064556270Z-CORE-CAST-DOC-COMMENTS-STILL-CONTAIN-91F72FD1: core/cast doc comments still contain generated boilerplate

## 概要

stdlib/core/cast.nepl still contains generated doc comment boilerplate such as 主な用途, 定義済み処理, 薄いラッパ, and generic move/rebind notes. These blocks duplicate the concrete cast/bitcast comments and do not document the conversion semantics.

## 対象

- `stdlib/core/cast.nepl; nodesrc/test_stdlib_cast_doc_no_boilerplate.js`

## 根拠

- `stdlib/core/cast.nepl` の先頭付近に生成テンプレート由来の `主な用途` / `定義済み処理` / `薄いラッパ` / move-rebind 注意が複数残っていた。
- 同じ位置に具体的な変換コメントも併存しており、doc debt が重複していた。

## 問題

stdlib/core/cast.nepl still contains generated doc comment boilerplate such as 主な用途, 定義済み処理, 薄いラッパ, and generic move/rebind notes. These blocks duplicate the concrete cast/bitcast comments and do not document the conversion semantics.

## 影響

The core conversion API documentation stays noisy and misleading, making it harder to review stdlib contracts for self-hosting and allowing generated placeholders to regress unnoticed.

## 修正方針

Replace the generated blocks with concrete Japanese nm-style comments that describe each cast/bitcast semantic and add a source policy regression that fails if the boilerplate phrases return to core/cast.nepl.

## 修正内容

- `cast` / `bitcast` のコメントを、数値変換、0/1 真偽値変換、下位 8 bit マスク、ビット列再解釈の意味が読める nm コメントへ整理した。
- `nodesrc/test_stdlib_cast_doc_no_boilerplate.js` を追加し、生成テンプレート文言の再混入と重要説明の欠落を検出するようにした。
- CI と `doc/testing.md` の source policy regressions に同テストを追加した。

## 検証

- `node nodesrc/test_stdlib_cast_doc_no_boilerplate.js`: pass
- `node nodesrc/tests.js -i stdlib/core/cast.nepl --no-tree -o tmp/cast-doc-boilerplate.json -j 1`: `total=1`, `passed=1`
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-cast-doc-boilerplate.json -j 4`: `total=418`, `passed=418`
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass（CRLF 変換 warning のみ）
