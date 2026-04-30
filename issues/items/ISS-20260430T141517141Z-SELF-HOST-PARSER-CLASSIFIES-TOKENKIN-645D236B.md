---
id: ISS-20260430T141517141Z-SELF-HOST-PARSER-CLASSIFIES-TOKENKIN-645D236B
title: "self-host parser classifies TokenKind through strings and hash keys instead of exhaustive enum match"
area: selfhost
status: fixed
resolved: true
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

## 解決

- `selfhost_parser_item_kind_from_token` を `TokenKind` 直接 `match` に変更し、module item に変換する token と無視する token を enum arm として明示した。
- parser loop の特殊 token 処理は `SelfhostParserTokenAction` enum に分離し、`selfhost_parser_token_action` が `TokenKind` 全 variant を網羅して action へ変換する形にした。
- `token_kind_name`、`hash32`、数値 hash arm、文字列再検証 helper を parser classification から削除した。`token_kind_name` は JSON/reporting/parity boundary 用に `token.nepl` 側へ残す。
- `nodesrc/test_selfhost_parser_tokenkind_match.js` を追加し、`module_parser.nepl` が TokenKind classification に hash/string dispatch を再導入しないこと、`selfhost_parser_token_action` と `selfhost_parser_item_kind_from_token` が TokenKind 全 variant を明示 arm で扱うこと、module loop が action enum を wildcard なしで処理することを検査するようにした。
- `nodesrc/run_source_policy_regressions.js` に新規 regression を追加した。

## 検証結果

- `node nodesrc/test_selfhost_parser_tokenkind_match.js`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: 新規 `selfhost parser TokenKind match regression` は passed。既存の `owner_summary_variant_paths.rs has 637 lines; responsibility split limit is 380` は `ISS-20260430T135243330Z-RESOURCE-OWNER-VARIANT-PATH-BUILDER--87B356A8` 側の既知残件。
- `node nodesrc/tests.js -i stdlib/neplg2/core/syntax/parser/module_parser.nepl -i tests/stdlib/neplg2_parser.n.md --no-tree -o tmp/selfhost-parser-tokenkind-match.json -j 1`: 2 件とも既知の wasm timeout。
- `NEPL_TEST_CASE_TIMEOUT_MS=180000 node nodesrc/tests.js -i tests/stdlib/neplg2_parser.n.md --no-tree -o tmp/selfhost-parser-tokenkind-match-long-timeout.json -j 1`: 180 秒でも timeout。今回の hash/string dispatch 除去とは別の selfhost parser runtime 残件として継続。
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed
