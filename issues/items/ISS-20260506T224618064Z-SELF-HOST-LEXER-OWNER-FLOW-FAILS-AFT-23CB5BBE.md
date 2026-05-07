---
id: ISS-20260506T224618064Z-SELF-HOST-LEXER-OWNER-FLOW-FAILS-AFT-23CB5BBE
title: "Self-host lexer owner flow fails after raw alias timeout fix"
area: selfhost
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-07
target: "stdlib/neplg2/core/syntax/lexer.nepl, nepl-core/src/resource"
---

# ISS-20260506T224618064Z-SELF-HOST-LEXER-OWNER-FLOW-FAILS-AFT-23CB5BBE: Self-host lexer owner flow fails after raw alias timeout fix

## 概要

After the compiler raw-alias summary no longer times out on an empty lex_all_with_file_id smoke case, Resource IR owner checking reached lex_all_loop and reported owner diagnostics for `SelfhostToken` / `LexDiagnostic` span fields. Investigation split the blocker into two root causes: self-host tokens stored owned lexeme strings in a Copy Vec element, and Resource IR owner summary treated ordinary `i32` aggregate fields as owner leaves.

## 対象

- `stdlib/neplg2/core/syntax/lexer.nepl, nepl-core/src/resource`

## 根拠

- `NEPL_COMPILE_STAGE_TIMING=1 cargo run -p nepl-cli -- --stdlib-root stdlib -i tmp/agent1_probe_lex_empty.nepl -o tmp/agent1_probe_lex_empty_after_token_range_out --emit wasm` after removing token lexeme ownership still failed on `SelfhostToken.span.end` and `LexDiagnostic.span.end`.
- The remaining diagnostics pointed at `Place { ... ty: TypeId(1) }`, i.e. ordinary `i32`, not an owned allocation. That showed Resource IR owner summary was conflating raw-address owner leaves with plain scalar fields.
- `SelfhostToken` was also a real stdlib issue: it was `Copy`, stored in `Vec<SelfhostToken>`, and contained `lexeme <str>`. This repeated the owned-payload-in-Copy-Vec pattern fixed earlier for import specs.

## 問題

`SelfhostToken` mixed a non-owning lexical identity with owned `str` storage, and Resource IR owner summary had an overbroad fallback where `TypeKind::I32` was always an owner leaf. This made ordinary spans and diagnostics look like free obligations and also inflated owner summary work.

## 影響

The empty lexer smoke case and lexer doctests could not compile with strict owner checking. The scalar-owner false positive also risked hiding real memory-safety issues by training later code to add unnecessary ownership workarounds around plain integers.

## 修正方針

- `SelfhostToken` を kind/span の range-only token に変更し、lexeme は `selfhost_token_lexeme source token` で消費境界だけ切り出す。
- lexer の keyword 判定は一時 `str` を使うが、token buffer へ保存せず `lex_consume_temp_str` で境界を閉じる。
- parser/module item への橋渡しは source を parser loop に渡し、token から必要時に lexeme を復元する。
- Resource IR owner summary は `MemPtr` leaf / `str` / owner-carrying aggregate だけを通常 owner leaf とし、裸 `i32` は Dealloc/Realloc を実際に扱う raw owner function の parameter seed に限定する。
- 古い「裸 i32 identity helper が raw owner を暗黙に転送する」テストは、技術的負債として逆向きの仕様へ更新した。

## 検証

- `cargo fmt -p nepl-core --check`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_does_not_treat_plain_i32_identity_as_owner_return -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_summary_does_not_treat_bool_parameters_as_owners -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_summary_does_not_treat_plain_i32_struct_fields_as_owners -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_applies_result_ok_raw_dealloc_consumption -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_applies_result_ok_mem_ptr_dealloc_consumption -- --nocapture`: passed
- `node nodesrc/test_selfhost_string_helpers_boundary.js`: passed
- `NEPL_COMPILE_STAGE_TIMING=1 cargo run -p nepl-cli -- --stdlib-root stdlib -i tmp/agent1_probe_lex_empty.nepl -o tmp/agent1_probe_lex_empty_after_final_out --emit wasm`: passed。`resource_owner_obligations` は約 1.4s、`resource_static_check` は約 16.5s。
- `trunk build`: passed
- `NEPL_TEST_CASE_TIMEOUT_MS=120000 node nodesrc/tests.js -i tmp/agent1_probe_lex_empty.n.md --no-tree --dist web/dist -o tmp/agent1_probe_lex_empty_after_range_dist.json -j 1 --assert-io`: passed
- `NEPL_TEST_CASE_TIMEOUT_MS=120000 node nodesrc/tests.js -i tests/stdlib/neplg2_lexer.n.md --no-tree --dist web/dist -o tmp/neplg2_lexer_range_only_tokens_after_dist.json -j 1 --assert-io`: passed, 13/13
- `node nodesrc/issues.js check`: passed
