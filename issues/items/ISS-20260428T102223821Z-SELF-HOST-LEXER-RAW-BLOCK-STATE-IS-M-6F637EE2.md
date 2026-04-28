---
id: ISS-20260428T102223821Z-SELF-HOST-LEXER-RAW-BLOCK-STATE-IS-M-6F637EE2
title: "self-host lexer raw block state is missing"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/neplg2/core/syntax/lexer.nepl, tests/stdlib/neplg2_lexer.n.md"
---

# ISS-20260428T102223821Z-SELF-HOST-LEXER-RAW-BLOCK-STATE-IS-M-6F637EE2: self-host lexer raw block state is missing

## 概要

TokenKind に WasmText / LlvmIrText は追加されたが、self-host lexer の lex_all_loop は #wasm: / #llvmir: 後の raw block state を保持しないため、raw block 本文を Rust lexer と同じ WasmText / LlvmIrText token stream として生成できない。

## 対象

- `stdlib/neplg2/core/syntax/lexer.nepl, tests/stdlib/neplg2_lexer.n.md`

## 根拠

- 未記入

## 問題

TokenKind に WasmText / LlvmIrText は追加されたが、self-host lexer の lex_all_loop は #wasm: / #llvmir: 後の raw block state を保持しないため、raw block 本文を Rust lexer と同じ WasmText / LlvmIrText token stream として生成できない。

## 影響

self-host parser parity で raw wasm / llvmir block の lexer 差分が parser 差分に混ざり、S1 parser 実装時に raw block directive の扱いを切り分けにくい。

## 修正方針

#wasm: / #llvmir: を読んだ行で pending raw mode と expected base indent を保持し、次行以降は base indent を満たす間 WasmText / LlvmIrText を生成する。dedent で raw mode を終了し、Rust lexer と同じ newline / indent / dedent span を維持する。

## 検証

Rust analyze_lex JSON と self-host lexer output を #wasm: / #llvmir: fixture で比較し、WasmText / LlvmIrText / Indent / Dedent / Newline の kind と span が一致することを確認する。

## 対応結果

- `lex_all_loop` に raw block state を追加し、`DirWasm` / `DirLlvmIr` を読んだ次行から pending raw mode を開始するようにした。
- raw block 開始時は現在の indent level + `#indent` 幅を base indent として `Indent` を生成し、base 以上の行を `WasmText` / `LlvmIrText` として token 化するようにした。
- base 未満へ dedent した行で raw mode を終了し、通常の offside rule へ戻るようにした。
- `tests/stdlib/neplg2_lexer.n.md` に doc / mlstr / `#wasm:` / `#llvmir:` を含む focused regression を追加した。

## 実行した検証

- `trunk build`: pass
- `node nodesrc/tests.js -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/neplg2-lexer-raw-block-after-build.json -j 1`: 12/12 passed
- `node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/neplg2-selfhost-raw-block-final.json -j 1`: 37/37 passed
- `node nodesrc/test_selfhost_lexer_rust_parity.js`: pass（2 fixtures / 79 tokens）
- `node nodesrc/issues.js check`: pass（files=255）
- `git diff --check HEAD`: pass（CRLF warning only）
