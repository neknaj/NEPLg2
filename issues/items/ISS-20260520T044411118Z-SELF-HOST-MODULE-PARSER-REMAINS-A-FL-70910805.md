---
id: ISS-20260520T044411118Z-SELF-HOST-MODULE-PARSER-REMAINS-A-FL-70910805
title: "self-host module parser remains a flat implementation file"
area: selfhost
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/neplg2/core/syntax/parser/module_parser.nepl; stdlib/neplg2/core/syntax/parser/module_parser/**; doc/neplg2/self_host_source_tree_layout_review_20260518.md"
---

# ISS-20260520T044411118Z-SELF-HOST-MODULE-PARSER-REMAINS-A-FL-70910805: self-host module parser remains a flat implementation file

## 概要

Selfhost module_parser.nepl keeps parser state, TokenKind action classification, diagnostics, module item classification, declaration header extraction, raw backend loop handling, parse entry points, and stage smoke in one file. This repeats the Rust parser.rs flat-file risk and makes parser/checker proof boundaries harder to audit.

## 対象

- `stdlib/neplg2/core/syntax/parser/module_parser.nepl; stdlib/neplg2/core/syntax/parser/module_parser/**; doc/neplg2/self_host_source_tree_layout_review_20260518.md`

## 根拠

- `doc/neplg2/self_host_source_tree_layout_review_20260518.md` は Rust 側 `parser.rs` の flat 構造を self-host 側へ移植しない方針を明記している。
- `stdlib/neplg2/core/syntax/parser/module_parser.nepl` は分割前に parser state、TokenKind action classification、diagnostic adapter、module item classification、declaration header extraction、raw backend loop、parse entry point を同じ file に持っていた。
- declaration header proof、abstraction、static-check proof boundary を今後追加する前に、parser の責務境界を file 単位で分ける必要があった。

## 問題

Selfhost module_parser.nepl keeps parser state, TokenKind action classification, diagnostics, module item classification, declaration header extraction, raw backend loop handling, parse entry points, and stage smoke in one file. This repeats the Rust parser.rs flat-file risk and makes parser/checker proof boundaries harder to audit.

## 影響

Further parser, declaration, abstraction, and static-check work will keep adding unrelated logic to module_parser.nepl, weakening exhaustive enum match policies and making source-level proof obligations harder to locate.

## 修正方針

Keep module_parser.nepl as a documentation/public facade, move implementation into module_parser/state.nepl, action.nepl, diagnostic.nepl, item_kind.nepl, declaration.nepl, loop.nepl, entry.nepl, and add a source-policy regression for the split.

## 検証

Run the parser split source-policy test, TokenKind match regression, parser report contract, module_parser source doctest, parser integration doctest, issue check, and git diff check.

## 修正内容

- `module_parser.nepl` を doctest と `pub #import` だけの public facade にした。
- 実装を `module_parser/state.nepl`、`action.nepl`、`diagnostic.nepl`、`item_kind.nepl`、`declaration.nepl`、`loop.nepl`、`entry.nepl` へ分割した。
- `TokenKind` から parser action / module item kind への分類は exhaustive match のまま維持し、文字列 / hash dispatch や wildcard arm へ戻していない。
- 元は同一 file 内 private だった declaration item construction は `declaration.nepl` に閉じ、`loop.nepl` は public `selfhost_parser_push_item` だけを呼ぶようにした。
- `nodesrc/selfhost_module_parser_sources.js` と `nodesrc/test_selfhost_module_parser_split_contract.js` を追加し、split 後 source policy を固定した。

## 検証結果

- `node nodesrc/test_selfhost_module_parser_split_contract.js`: pass
- `node nodesrc/test_selfhost_parser_tokenkind_match.js`: pass
- `node nodesrc/test_selfhost_parser_report_contract.js`: pass
- `node nodesrc/tests.js -i stdlib/neplg2/core/syntax/parser/module_parser.nepl --no-tree -o tmp/agent1-module-parser-split-core.json -j 1 --dist web/dist --assert-io`: total=1, passed=1
- `node nodesrc/tests.js -i tests/stdlib/neplg2_parser.n.md --no-tree -o tmp/agent1-module-parser-split-parser.json -j 1 --dist web/dist --assert-io`: total=1, passed=1
