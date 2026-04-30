---
id: ISS-20260430T141517141Z-SELF-HOST-PARSER-CLASSIFIES-TOKENKIN-645D236B
title: "self-host parser classifies TokenKind through strings and hash keys instead of exhaustive enum match"
area: selfhost
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-30
updated: 2026-04-30
target: "stdlib/neplg2/core/syntax/parser/module_parser.nepl, stdlib/neplg2/core/syntax/token.nepl"
---

# ISS-20260430T141517141Z-SELF-HOST-PARSER-CLASSIFIES-TOKENKIN-645D236B: self-host parser classifies TokenKind through strings and hash keys instead of exhaustive enum match

## 概要

The self-host module parser receives TokenKind but converts it to token_kind_name, hashes the string, and dispatches on numeric hash arms. This bypasses enum-based exhaustiveness checking and leaves parser evolution dependent on string/hash constants.

## 対象

- `stdlib/neplg2/core/syntax/parser/module_parser.nepl, stdlib/neplg2/core/syntax/token.nepl`

## 根拠

- `stdlib/neplg2/core/syntax/parser/module_parser.nepl` has `selfhost_parser_item_kind_from_token` call `selfhost_parser_item_kind_from_name token_kind_name kind depth`.
- `selfhost_parser_item_kind_from_name` dispatches on `selfhost_parser_string_match_key` / `hash32` numeric arms, then validates with string equality.
- `stdlib/neplg2/core/syntax/token.nepl` already defines `TokenKind` as an enum, so the parser can classify with direct exhaustive `match kind` instead.

## 問題

This design turns a statically known TokenKind into a string/hash protocol at the parser boundary. Adding or renaming token kinds can silently miss parser handling until runtime tests catch it, and the numeric hash arms obscure the intended grammar mapping. It directly conflicts with the project policy that safety-critical finite state should use enum values and match exhaustiveness rather than numbers or strings.

## 影響

Self-host parser parity becomes harder to audit, and future parser stages may copy the same hash-string pattern for grammar decisions. That weakens static checking exactly where self-host needs reliable enum/match coverage for Rust parity and bootstrap diagnostics.

## 修正方針

Refactor parser item classification to match directly on TokenKind. Keep token_kind_name only for JSON/reporting boundaries. If multiple token groups need sharing, introduce typed helper functions that accept TokenKind and return Option<SelfhostModuleItemKind>, with exhaustive match arms for all TokenKind variants. Add a source policy regression that rejects token_kind_name/hash32 dispatch inside parser classification.

## 検証

Use gh Actions after implementation to confirm selfhost/stdlib doctest status. For pre-commit implementation checks, run focused selfhost parser doctests, selfhost lexer/parser parity tests, the new source policy regression, node nodesrc/issues.js check, and git diff --check.
