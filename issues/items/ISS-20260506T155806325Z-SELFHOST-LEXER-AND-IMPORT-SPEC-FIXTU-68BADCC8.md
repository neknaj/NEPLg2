---
id: ISS-20260506T155806325Z-SELFHOST-LEXER-AND-IMPORT-SPEC-FIXTU-68BADCC8
title: "Selfhost lexer and import spec fixtures drift under strict static checks"
area: selfhost
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-06
target: "stdlib/neplg2/core/syntax/lexer.nepl, stdlib/neplg2/core/module/import_spec.nepl, tests/stdlib/neplg2_import_spec.n.md"
---

# ISS-20260506T155806325Z-SELFHOST-LEXER-AND-IMPORT-SPEC-FIXTU-68BADCC8: Selfhost lexer and import spec fixtures drift under strict static checks

## 概要

After direct string submodule imports, focused selfhost doctests no longer stop at undefined string facade names. They expose strict checker failures: lex_stack_drop_top returns a Vec constructor expression with stack/type mismatch, and import_spec doctests leak the SelfhostImportSpec path/alias str owners.

## 対象

- `stdlib/neplg2/core/syntax/lexer.nepl, stdlib/neplg2/core/module/import_spec.nepl, tests/stdlib/neplg2_import_spec.n.md`

## 根拠

- 未記入

## 問題

After direct string submodule imports, focused selfhost doctests no longer stop at undefined string facade names. They expose strict checker failures: lex_stack_drop_top returns a Vec constructor expression with stack/type mismatch, and import_spec doctests leak the SelfhostImportSpec path/alias str owners.

## 影響

Selfhost module graph and import-spec behavior cannot be validated reliably. These failures block selfhost progress independently of the Rust borrow-checker work and may hide parser/import regressions behind stale fixture ownership code.

## 修正方針

Fix the selfhost lexer Vec construction/stack discipline and update import_spec doctests or APIs so returned SelfhostImportSpec string owners are consumed or freed explicitly. Keep enum/match coverage and do not disable ResourceIR owner diagnostics.

## 検証

Run neplg2_import_spec, module graph/loader, lexer focused doctests and source-policy selfhost boundary checks.
