---
id: ISS-20260506T203121413Z-COMPILER-STATIC-CHECKER-TIMES-OUT-ON-5B942F4A
title: "Compiler static checker times out on self-host lexer lex_all owner/offside flow"
area: compiler
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/src/resource, stdlib/neplg2/core/syntax/lexer.nepl, nodesrc/tests.js"
---

# ISS-20260506T203121413Z-COMPILER-STATIC-CHECKER-TIMES-OUT-ON-5B942F4A: Compiler static checker times out on self-host lexer lex_all owner/offside flow

## 概要

On origin/main `5a8515ec`, an empty self-host lexer smoke test that only calls `lex_all_with_file_id "" 0` still times out in compile phase. Raising `NEPL_TEST_CASE_TIMEOUT_MS` to 240000ms also timed out before diagnostics. A local stdlib-only refactor experiment did not remove the timeout, while import-only probes for the split lexer submodules completed quickly. The blocker is therefore the compiler static/resource analysis of the owner-bearing `lex_all` / offside flow, not parser/runtime execution.

## 対象

- `nepl-core/src/resource, stdlib/neplg2/core/syntax/lexer.nepl, nodesrc/tests.js`

## 根拠

- `node nodesrc/tests.js -i tmp/probe_lex_empty.n.md --no-tree -o tmp/probe_lex_empty_after_remote_resource_fixes.json -j 1` on `5a8515ec` still timed out at compile phase after 60000ms.
- `NEPL_TEST_CASE_TIMEOUT_MS=240000 node nodesrc/tests.js -i tmp/probe_lex_empty.n.md --no-tree -o tmp/probe_lex_empty_long_timeout.json -j 1` also timed out at compile phase after 240000ms.
- Local stdlib-only experiments split lexer internals into `types` / `scan` / `keyword` / `reader` / `offside`, replaced raw-mode `i32` sentinels with enum state, removed calls to token-wide predicate helpers from the lexer loop, and rewrote the offside loop away from mutual recursion. The `lex_all_with_file_id` call still timed out.
- Import-only probes for the split submodules completed in about 5-6 seconds, which indicates that the timeout is triggered by static/resource analysis of the called owner/offside flow rather than file parsing or module import alone.

## 問題

The compiler static checker/resource analysis does not finish within the test budget when checking the self-host lexer `lex_all_with_file_id` owner/offside flow. The reproducer does not need source content: an empty input is enough. The checked path owns and transforms `Vec<SelfhostToken>` plus an indent stack through `Result<Vec<SelfhostToken>, LexDiagnostic>` branches, offside state, EOF/Dedent insertion, and lexical error conversion.

## 影響

Self-host parser, loader, and module graph doctests cannot provide CI signal because any path that calls lex_all_with_file_id compiles past the test budget. Stdlib-only refactoring cannot safely close the blocker until Resource IR/static checker handles the lexer owner/offside control flow within budget.

## 修正方針

Profile Resource IR/static checker on the lex_all_with_file_id empty-source probe, especially owner Vec state through recursive/looping offside flow, Result<Vec<SelfhostToken>,LexDiagnostic> branches, and token buffer/indent stack owner transitions. Add a focused compiler regression using the lexer smoke case or a minimized owner/offside reproducer, then update the self-host lexer issue once compile phase is below the default budget.

## 検証

Run node nodesrc/tests.js -i tmp/probe_lex_empty.n.md --no-tree -o tmp/probe_lex_empty_after_compiler_fix.json -j 1 under the default 60000ms timeout, then run stdlib/neplg2/core/syntax/parser/module_parser.nepl, stdlib/neplg2/core/module/loader.nepl, and stdlib/neplg2/core/module/graph.nepl focused doctests.
