---
id: ISS-20260519T221017627Z-SELF-HOST-MODULE-AST-DROPS-DECLARATI-7F9E1FD2
title: "self-host module AST drops declaration header evidence before proof"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-19
updated: 2026-05-20
target: stdlib/neplg2/core/syntax/ast/module_ast.nepl
---

# ISS-20260519T221017627Z-SELF-HOST-MODULE-AST-DROPS-DECLARATI-7F9E1FD2: self-host module AST drops declaration header evidence before proof

## 概要

SelfhostModuleItem stores declaration items as only the keyword token span and lexeme, so the self-host checker cannot pass declaration header evidence into the generic proof solver. This blocks later type, trait, effect, and Resource IR checks from proving properties from source facts.

## 対象

- `stdlib/neplg2/core/syntax/ast/module_ast.nepl`

## 根拠

- `SelfhostModuleItem` は declaration item を `kind`、keyword token span、lexeme だけで保持しており、parser が観測した declaration header 全体の span、visibility、head token の typed evidence を失っていた。
- `check/module.nepl` は今後の declaration well-formedness を検査するために lexeme の文字列再走査か checker-local rule を増やすしかなく、`core/proof/` に fact / obligation / evidence / refutation を集約する Stage 6 方針と衝突していた。
- `doc/neplg2/static_check_complexity_reduction_plan.md` の Stage 6 は、静的検査を stdlib 名や個別 module allowlist ではなく source evidence と typed proof solver へ寄せる方針であり、declaration header も同じ境界に載せる必要があった。

## 問題

SelfhostModuleItem stores declaration items as only the keyword token span and lexeme, so the self-host checker cannot pass declaration header evidence into the generic proof solver. This blocks later type, trait, effect, and Resource IR checks from proving properties from source facts.

## 影響

Declaration well-formedness would have to be reconstructed with ad hoc string scans or checker-local logic, which conflicts with the Stage 6 static-check redesign and the enum/match-based proof policy.

## 修正方針

Add typed declaration header evidence to the module AST, have module_parser populate it from the token stream, add a generic proof fact/obligation/refutation for declaration header availability, and make module checker validate declaration items through the shared proof solver.

## 検証

- `SelfhostModuleDeclarationKind` / `SelfhostModuleDeclarationVisibility` / `SelfhostModuleDeclarationHeadKind` / `SelfhostModuleDeclarationHeader` を AST に追加し、parser が declaration item を `selfhost_module_item_new_with_declaration` で構築するようにした。
- `SelfhostModuleDeclarationFact`、`SelfhostProofObligation::ModuleDeclarationHeaderAvailable`、typed evidence / refutation を追加し、proof solver が header 欠落と kind/span/head 不整合を enum match で拒否するようにした。
- `check/module.nepl` は declaration item を `selfhost_proof_module_declaration_header` へ渡し、`checker.module.declaration_header_missing` / `checker.module.declaration_header_invalid` へ変換する責務だけを持つ。
- `node nodesrc/test_selfhost_proof_entry_contract.js`: pass
- `node nodesrc/test_selfhost_diag_code_enum.js`: pass
- `node nodesrc/tests.js -i tests/stdlib/neplg2_parser.n.md -o $env:TEMP\neplg2_parser_test.json`: total=21, passed=21
- `node nodesrc/tests.js -i tests/stdlib/neplg2_proof.n.md -o $env:TEMP\neplg2_proof_test.json`: total=24, passed=24
- `node nodesrc/tests.js -i tests/stdlib/neplg2_checker.n.md -o $env:TEMP\neplg2_checker_test_j1.json -j 1`: total=24, passed=24
- `node nodesrc/tests.js -i stdlib/neplg2/core/syntax/ast/module_ast.nepl -i stdlib/neplg2/core/syntax/parser/module_parser.nepl -i stdlib/neplg2/core/proof.nepl -i stdlib/neplg2/core/check/module.nepl -o $env:TEMP\neplg2_selfhost_impl_test.json -j 1`: total=24, passed=24
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass
