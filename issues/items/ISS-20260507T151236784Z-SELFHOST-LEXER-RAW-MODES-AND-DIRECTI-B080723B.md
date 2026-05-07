---
id: ISS-20260507T151236784Z-SELFHOST-LEXER-RAW-MODES-AND-DIRECTI-B080723B
title: "Selfhost lexer raw modes and directives bypass enum match coverage"
area: selfhost
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-05-07
updated: 2026-05-08
target: "stdlib/neplg2/core/syntax/lexer.nepl, tests/stdlib/neplg2_lexer.n.md, nodesrc/test_selfhost_lexer_rust_parity.js"
---

# ISS-20260507T151236784Z-SELFHOST-LEXER-RAW-MODES-AND-DIRECTI-B080723B: Selfhost lexer raw modes and directives bypass enum match coverage

## 概要

The selfhost lexer stores raw block state as i32 sentinels: raw_mode and pending_raw_mode use 0/1/2, lex_token_pending_raw_mode returns numeric mode values, and lex_raw_kind maps non-1 values to LlvmIrText. Directive classification is also a deep nested if chain instead of a finite enum or hash+match classifier.

## 対象

- `stdlib/neplg2/core/syntax/lexer.nepl, tests/stdlib/neplg2_lexer.n.md, nodesrc/test_selfhost_lexer_rust_parity.js`

## 根拠

- 未記入

## 問題

The selfhost lexer stores raw block state as i32 sentinels: raw_mode and pending_raw_mode use 0/1/2, lex_token_pending_raw_mode returns numeric mode values, and lex_raw_kind maps non-1 values to LlvmIrText. Directive classification is also a deep nested if chain instead of a finite enum or hash+match classifier.

## 影響

Raw block handling and directive additions are memory/type-safety-adjacent parser inputs. With numeric modes, the static checker cannot enforce exhaustive handling when new raw modes or directive kinds are added, and an unexpected numeric value silently becomes LlvmIrText. The directive chain also repeats the pattern previously rejected for keyword classification.

## 修正方針

Introduce a SelfhostLexerRawMode enum and, if needed, separate Pending/Active raw state records so raw mode transitions are match-exhaustive. Replace directive classification with a hash/key + match + str_starts_with_at verification table following the keyword and CLI arg classifier pattern. Keep #indent/#wasm/#llvmir parity with Rust lexer.

## 検証

Add source-policy tests rejecting i32 raw_mode/pending_raw_mode state and lex_raw_kind fallback behavior, plus regression fixtures for #wasm, #llvmir, #indent, #if[target], unknown directives, and raw block dedent. Keep rust/selfhost lexer parity checks covering directive and raw text token kinds/spans.

## 2026-05-08 Agent 1 対応

`raw_mode` / `pending_raw_mode` を `SelfhostLexerRawMode::{None,Wasm,LlvmIr}` に置き換え、raw block active 判定と raw token kind 変換を enum match で網羅する形にした。これにより、不明な i32 値が `LlvmIrText` へ落ちる旧 fallback は存在しない。

directive token 化は `SelfhostLexerDirectiveKind` に分類してから `lex_directive_token` で match する構造に変更した。byte-key bucket は候補削減だけを担当し、最終的な token kind への対応は `SelfhostLexerDirectiveKind` の全 variant を明示した match で固定している。`#if[target=...]` / `#if[profile=...]` の prefix 検証は `lex_directive_kind_if_prefix` に集約し、`str_starts_with_at` facade を使う既存 boundary policy も helper 構造へ更新した。

追加した回帰:

- `nodesrc/test_selfhost_lexer_raw_mode_directive_enum.js`
- `nodesrc/run_source_policy_regressions.js` への登録
- `nodesrc/test_selfhost_string_helpers_boundary.js` の directive prefix helper policy 更新

検証:

- `node nodesrc/test_selfhost_lexer_raw_mode_directive_enum.js`: passed
- `node nodesrc/test_selfhost_string_helpers_boundary.js`: passed
- `node nodesrc/test_selfhost_lexer_rust_parity.js`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed
- `git diff --check`: passed
- `node nodesrc/tests.js -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/agent1-selfhost-lexer-raw-mode-enum.json -j 1 --dist web/dist`: 13/13 passed。初回 300 秒 timeout は 10/13 passed の進行中で、各 compile が約 39 秒かかるため timeout 設定不足だった。
