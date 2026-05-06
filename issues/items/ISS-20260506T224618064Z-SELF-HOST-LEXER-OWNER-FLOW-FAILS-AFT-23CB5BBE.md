---
id: ISS-20260506T224618064Z-SELF-HOST-LEXER-OWNER-FLOW-FAILS-AFT-23CB5BBE
title: "Self-host lexer owner flow fails after raw alias timeout fix"
area: selfhost
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-06
target: "stdlib/neplg2/core/syntax/lexer.nepl, nepl-core/src/resource"
---

# ISS-20260506T224618064Z-SELF-HOST-LEXER-OWNER-FLOW-FAILS-AFT-23CB5BBE: Self-host lexer owner flow fails after raw alias timeout fix

## 概要

After the compiler raw-alias summary no longer times out on an empty lex_all_with_file_id smoke case, Resource IR owner checking reaches lex_all_loop and lex_all_with_file_id and reports resource.owner.maybe_leak for SelfhostToken string/span fields plus resource.owner.use_after_move for the initial indent stack push result. The previous timeout hid these diagnostics.

## 対象

- `stdlib/neplg2/core/syntax/lexer.nepl, nepl-core/src/resource`

## 根拠

- `NEPL_COMPILE_STAGE_TIMING=1 cargo run -p nepl-cli -- --stdlib-root stdlib -i tmp/agent1_probe_lex_empty.nepl -o tmp/agent1_probe_lex_empty_out --emit wasm` after `ISS-20260506T203121413Z-COMPILER-STATIC-CHECKER-TIMES-OUT-ON-5B942F4A` no longer timed out.
- `NEPL_TEST_CASE_TIMEOUT_MS=60000 node nodesrc/tests.js -i tmp/agent1_probe_lex_empty.n.md --no-tree -o tmp/agent1_probe_lex_empty_after_seed_gate_wasm.json -j 1 --assert-io` also no longer timed out; it reported compile diagnostics after `compile_ms=37347`.
- The same run reached `resource_owner_obligations` and reported:
  - `resource.owner.maybe_leak` in `lex_all_loop__...` for `raw_token` / `token` projections that include `SelfhostToken` span / lexeme fields.
  - `resource.owner.use_after_move` in `lex_all_with_file_id__...` at the initial `push<i32> stack0 0` result handling.
  - `resource.owner.maybe_leak` in the empty smoke test `main` for the `LexDiagnostic` owner path.
- This is a separate blocker from the compiler timeout: the compiler now produces diagnostics within budget, but lexer behavioral doctests still cannot execute.

## 問題

After the compiler raw-alias summary no longer times out on an empty lex_all_with_file_id smoke case, Resource IR owner checking reaches lex_all_loop and lex_all_with_file_id and reports resource.owner.maybe_leak for SelfhostToken string/span fields plus resource.owner.use_after_move for the initial indent stack push result. The previous timeout hid these diagnostics.

## 影響

Self-host lexer/parser/loader/module graph doctests still cannot run as behavioral CI signal after the compiler timeout is removed. The remaining blocker must be resolved without weakening owner diagnostics, because token lexeme/span string owners and Vec owner transfers are part of the memory-safety contract.

## 修正方針

Trace whether the diagnostics are real stdlib owner-transfer bugs or Resource owner summary false positives around Result<Vec<SelfhostToken>,LexDiagnostic>, Vec push failure branches, and Copy token/span fields. Fix the owner transfer model or self-host API shape at the root; do not suppress resource.owner.maybe_leak/use_after_move.

## 検証

Run the empty lex_all_with_file_id smoke case and tests/stdlib/neplg2_lexer.n.md under the default timeout; require compile and runtime pass without resource.owner.maybe_leak or resource.owner.use_after_move. Then rerun parser module_parser, loader, and graph focused doctests.
