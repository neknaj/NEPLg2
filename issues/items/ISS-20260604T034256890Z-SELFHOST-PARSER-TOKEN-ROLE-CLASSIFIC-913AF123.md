---
id: ISS-20260604T034256890Z-SELFHOST-PARSER-TOKEN-ROLE-CLASSIFIC-913AF123
title: "selfhost parser token role classification is duplicated across large match tables"
area: selfhost
status: fixed
resolved: true
priority: P2
type: maintenance
created: 2026-06-04
updated: 2026-06-04
target: "stdlib/neplg2/core/syntax/parser/module_parser/token_role.nepl, stdlib/neplg2/core/syntax/parser/module_parser/token_role_header.nepl, stdlib/neplg2/core/syntax/parser/module_parser/action.nepl, stdlib/neplg2/core/syntax/parser/module_parser/declaration.nepl, stdlib/neplg2/core/syntax/parser/module_parser/header_boundary.nepl, stdlib/neplg2/core/syntax/parser/module_parser/item_kind.nepl, nodesrc/test_selfhost_parser_tokenkind_match.js"
---

# ISS-20260604T034256890Z-SELFHOST-PARSER-TOKEN-ROLE-CLASSIFIC-913AF123: selfhost parser token role classification is duplicated across large match tables

## 概要

Subagent audit found visibility, declaration head kind, and item kind classification distributed across separate match tables. This conflicts with the Zenn guidance around DAG, directory hierarchy, and static verification of classification responsibilities.

## 対象

- `stdlib/neplg2/core/syntax/parser/module_parser/declaration.nepl, stdlib/neplg2/core/syntax/parser/module_parser/item_kind.nepl`

## 根拠

- `TokenKind` の parser role 分類が `action.nepl`、`item_kind.nepl`、`declaration.nepl`、`header_boundary.nepl` に重複していた。
- `KwFn`、`KwPub`、`Ident`、`Dot`、raw backend directive などの意味づけが別々の match table で管理されると、NEPLg2.1 構文移行時に片方だけ更新される危険がある。
- Zenn 方針の enum による静的検査、責務分割、DAG 化に合わせるには、`TokenKind -> SelfhostParserTokenRole` を唯一の分類 authority にし、下流は projection として扱う必要がある。

## 問題

Subagent audit found visibility, declaration head kind, and item kind classification distributed across separate match tables. This conflicts with the Zenn guidance around DAG, directory hierarchy, and static verification of classification responsibilities.

## 影響

Adding or migrating syntax tokens requires editing several tables, making it easy for parser, syntax highlight, and diagnostic behavior to drift.

## 修正方針

Introduce a token role enum/helper table and route visibility/head/item classification through the shared contract, with source policy guarding duplicated classification.

## 対応

- `SelfhostParserTokenRole` と `selfhost_parser_token_role` を追加し、module parser が必要とする token role を1か所で網羅分類した。
- loop action、module item kind、declaration head kind、visibility は role から射影する形に変更した。
- declaration keyword の block depth 0 制約と statement boundary 制約は文脈依存なので、role table へ吸収せず既存の parser context 側に残した。
- `selfhost_parser_token_action`、`selfhost_parser_item_kind_from_token`、`selfhost_parser_declaration_head_kind`、`selfhost_parser_declaration_visibility` が `TokenKind` を直接 match しないことを source policy で検査するようにした。

## 検証

- pass: `node nodesrc\test_selfhost_module_parser_split_contract.js`
- pass: `node nodesrc\test_selfhost_parser_tokenkind_match.js`
- pass: `node nodesrc\test_selfhost_parser_current_syntax_boundary.js`
- pass: `node nodesrc\test_selfhost_parser_report_contract.js`
- pass: `node nodesrc\test_selfhost_parser_invalid_state_contract.js`
- pass: `node nodesrc\test_selfhost_diag_code_enum.js`
- pass: `node nodesrc\tests.js -i tests\stdlib\neplg2_parser.n.md --no-tree -o tmp\selfhost-parser-token-role-focused.json -j 1 --assert-io --dist web\dist`
