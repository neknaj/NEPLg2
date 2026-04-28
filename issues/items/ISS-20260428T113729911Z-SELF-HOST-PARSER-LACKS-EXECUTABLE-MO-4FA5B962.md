---
id: ISS-20260428T113729911Z-SELF-HOST-PARSER-LACKS-EXECUTABLE-MO-4FA5B962
title: "self-host parser lacks executable ModuleAst and raw block items"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/neplg2/core/syntax/ast/module_ast.nepl, stdlib/neplg2/core/syntax/parser/module_parser.nepl, tests/stdlib/neplg2_parser.n.md"
---

# ISS-20260428T113729911Z-SELF-HOST-PARSER-LACKS-EXECUTABLE-MO-4FA5B962: self-host parser lacks executable ModuleAst and raw block items

## 概要

module_ast.nepl and module_parser.nepl are still Stage 0 marker APIs, so the lexer token stream cannot be parsed into a ModuleAst. In particular #wasm: and #llvmir: raw directive blocks now produce DirWasm/DirLlvmIr plus raw text tokens, but parser has no item representation or validation path for them.

## 対象

- `stdlib/neplg2/core/syntax/ast/module_ast.nepl, stdlib/neplg2/core/syntax/parser/module_parser.nepl, tests/stdlib/neplg2_parser.n.md`

## 根拠

- 未記入

## 問題

module_ast.nepl and module_parser.nepl are still Stage 0 marker APIs, so the lexer token stream cannot be parsed into a ModuleAst. In particular #wasm: and #llvmir: raw directive blocks now produce DirWasm/DirLlvmIr plus raw text tokens, but parser has no item representation or validation path for them.

## 影響

S1 parser parity cannot start from Rust AST JSON, raw backend bodies cannot be preserved by the self-host frontend, and future module loader/checker work has no stable AST contract to consume.

## 修正方針

Introduce a small executable ModuleAst contract for top-level directive/raw/text items, implement a token-stream parser that preserves doc comments, mlstr lines, and wasm/llvm raw block text, and add focused doctests that exercise the new raw block parser path.

## 検証

trunk build; node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_parser.n.md --no-tree; node nodesrc/issues.js check

## 2026-04-28 修正

- `SelfhostModuleAst` / `SelfhostModuleItem` / `SelfhostModuleItemKind` を追加し、parser が保持する最初の module item stream を実行可能な所有 AST にした。
- `selfhost_parse_module_tokens` / `selfhost_parse_module_source` を追加し、lexer token stream から doc comment、mlstr line、directive、top-level 宣言開始、`#wasm:` / `#llvmir:` raw block text を item 化できるようにした。
- raw backend block については pending / active raw mode を enum で管理し、`WasmText` / `LlvmIrText` が対応する raw mode の外へ現れた場合は diagnostic にする。
- `tests/stdlib/neplg2_parser.n.md` に、関数本体内の `#if[target=wasm]` / `#wasm:` と `#if[target=llvm]` / `#llvmir:` を parser item stream として確認する回帰テストを追加した。
- parser module は selfhost lexer / AST / hash dispatch を含むため、focused doctest は既定の 20 秒 case timeout を超えることがある。検証では `NEPL_TEST_CASE_TIMEOUT_MS=60000` を明示した。
