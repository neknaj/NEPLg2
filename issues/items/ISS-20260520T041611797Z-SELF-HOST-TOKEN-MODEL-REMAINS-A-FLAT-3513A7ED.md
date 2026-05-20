---
id: ISS-20260520T041611797Z-SELF-HOST-TOKEN-MODEL-REMAINS-A-FLAT-3513A7ED
title: "self-host token model remains a flat implementation file"
area: selfhost
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/neplg2/core/syntax/token.nepl, stdlib/neplg2/core/syntax/token/**, doc/neplg2/self_host_source_tree_layout_review_20260518.md"
---

# ISS-20260520T041611797Z-SELF-HOST-TOKEN-MODEL-REMAINS-A-FLAT-3513A7ED: self-host token model remains a flat implementation file

## 概要

Self-host token.nepl still keeps TokenKind, SelfhostToken value model, lexeme slicing, JSON/parity name mapping, four token predicates, and stage smoke API in one large file. The predicate helpers repeat exhaustive TokenKind matches and make later parser/lexer changes easy to append to the flat facade.

## 対象

- `stdlib/neplg2/core/syntax/token.nepl, stdlib/neplg2/core/syntax/token/**, doc/neplg2/self_host_source_tree_layout_review_20260518.md`

## 根拠

- `doc/neplg2/self_host_source_tree_layout_review_20260518.md` は `core/syntax/token.nepl` を P2 の分割対象として挙げ、`syntax/token/` 配下に kind、token value、display/name、directive classification を分ける方針を明記していた。
- 変更前の `token.nepl` は `TokenKind` enum、`SelfhostToken` value、lexeme slicing、stable name mapping、EOF/error/newline/expression-start predicates、stage smoke API を 1 file に持っていた。
- predicate helper はそれぞれ `TokenKind` 全 variant を exhaustive match しており、静的検査には良い一方で file size と責務境界の観点では token model と parser-facing predicate が混在していた。

## 問題

Self-host token.nepl still keeps TokenKind, SelfhostToken value model, lexeme slicing, JSON/parity name mapping, four token predicates, and stage smoke API in one large file. The predicate helpers repeat exhaustive TokenKind matches and make later parser/lexer changes easy to append to the flat facade.

## 影響

Leaving token model flat weakens source-tree review for lexer/parser work, makes source policy point at one oversized file, and risks mixing reporting name conversion with parser-facing token predicates.

## 修正方針

Keep syntax/token.nepl as a doctest-bearing implementation-free facade and split token responsibilities into kind, value, name, predicate modules, and stage0. Preserve exhaustive enum matches and update source policy tests to read split sources.

## 検証

Run token split source policy, parser TokenKind match policy, string helper boundary policy, focused token doctest, issues check, and diff whitespace check.

## 対応結果

- `core/syntax/token.nepl` は doctest を保持する implementation-free facade にした。
- 実装は `token/kind.nepl`、`value.nepl`、`name.nepl`、`predicate/eof.nepl`、`predicate/error.nepl`、`predicate/newline.nepl`、`predicate/expr_start.nepl`、`stage0.nepl` へ分割した。
- `TokenKind` enum と predicate helper の exhaustive match は維持した。
- token doctest は `core/math` の偶発的な推移 import に依存しないよう、`and` のための import を明示した。
- `nodesrc/selfhost_token_sources.js` と `nodesrc/test_selfhost_token_split_contract.js` を追加した。

## 検証結果

- `node nodesrc/test_selfhost_token_split_contract.js`: passed
- `node nodesrc/test_selfhost_parser_tokenkind_match.js`: passed
- `node nodesrc/test_selfhost_string_helpers_boundary.js`: passed
- `node nodesrc/test_selfhost_lexer_raw_mode_directive_enum.js`: passed
- `node nodesrc/tests.js -i stdlib/neplg2/core/syntax/token.nepl --no-tree -o tmp/agent1-token-split-core.json -j 1 --dist web/dist --assert-io`: 1/1 passed
- `node nodesrc/tests.js -i stdlib/neplg2/core/syntax/parser/module_parser.nepl --no-tree -o tmp/agent1-token-split-parser-module.json -j 1 --dist web/dist --assert-io`: 1/1 passed
- `node nodesrc/tests.js -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/agent1-token-split-lexer.json -j 1 --dist web/dist --assert-io`: 13/13 passed
