---
id: ISS-20260427T132142587Z-ALLOC-STRING-DOC-COMMENTS-STILL-CONT-C037036C
title: "alloc/string doc comments still contain generated boilerplate"
area: stdlib
status: verified
resolved: true
priority: P2
type: doc
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/string.nepl; nodesrc/test_stdlib_string_doc_no_boilerplate.js"
source: ISS-20260426T060358681Z-STDLIB-DOC-COMMENTS-STILL-CONTAIN-GE-2D7384D1
---

# ISS-20260427T132142587Z-ALLOC-STRING-DOC-COMMENTS-STILL-CONT-C037036C: alloc/string doc comments still contain generated boilerplate

## 概要

stdlib/alloc/string.nepl still contains generated doc comment boilerplate such as 主な用途, 定義済み処理, 薄いラッパ, and generic move/rebind notes. These comments obscure byte-length vs character semantics, Result-returning allocation paths, StringBuilder ownership, slicing/splitting byte boundaries, and numeric conversion overflow/radix behavior.

## 対象

- `stdlib/alloc/string.nepl; nodesrc/test_stdlib_string_doc_no_boilerplate.js`

## 根拠

- `rg` で `主な用途` / `定義済み処理` / `薄いラッパ` / `再利用時は束縛し直` が `stdlib/alloc/string.nepl` の public API コメントに多数残っていることを確認した。
- `to_i32` や StringBuilder の既存コメントには、現在の overflow check や一括 allocation 実装とずれた説明も残っている。

## 問題

stdlib/alloc/string.nepl still contains generated doc comment boilerplate such as 主な用途, 定義済み処理, 薄いラッパ, and generic move/rebind notes. These comments obscure byte-length vs character semantics, Result-returning allocation paths, StringBuilder ownership, slicing/splitting byte boundaries, and numeric conversion overflow/radix behavior.

## 影響

String is self-host critical for lexer, parser, diagnostics, and JSON/markdown emission. Placeholder comments make the API contracts harder to review and allow compiler-workaround style docs to regress unnoticed.

## 修正方針

Replace the generated blocks with concrete Japanese nm-style comments for the public string APIs, including allocation failure behavior and byte-index constraints. Add a source policy regression that fails if boilerplate phrases return to stdlib/alloc/string.nepl.

## 修正内容

- `len` / `concat_result` / `StringBuilder` / `str_slice_result` / `str_split_result` / 数値変換 / float 変換 / `find` のコメントを、実装契約に基づく日本語 nm-style コメントへ置き換えた。
- `StringBuilder` の build 計算量説明を現在の一括 allocation 実装に合わせ、`to_i32` の説明を range check 付き実装に合わせた。
- `nodesrc/test_stdlib_string_doc_no_boilerplate.js` を追加し、生成テンプレート文言の再混入と重要契約説明の欠落を検出するようにした。
- CI と `doc/testing.md` の source policy regressions に同テストを追加した。
- レビューで見つけた `str_slice_result` の UTF-8 境界問題と `to_f64` の parser 状態問題を別 issue として追加し、Discord へ報告した。

## 検証

- `node nodesrc/test_stdlib_string_doc_no_boilerplate.js`: pass
- `node nodesrc/test_stdlib_string_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i stdlib/alloc/string.nepl -i stdlib/tests/string.n.md -i tests/stdlib/string_numeric_overflow.n.md -i tests/stdlib/selfhost_req.n.md --no-tree -o tmp/string-doc-boilerplate-focused.json -j 1`: `total=27`, `passed=27`
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-string-doc-boilerplate.json -j 4`: `total=418`, `passed=418`
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass（CRLF 変換 warning のみ）
