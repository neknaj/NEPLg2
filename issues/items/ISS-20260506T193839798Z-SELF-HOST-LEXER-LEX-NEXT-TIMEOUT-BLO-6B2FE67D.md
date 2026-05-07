---
id: ISS-20260506T193839798Z-SELF-HOST-LEXER-LEX-NEXT-TIMEOUT-BLO-6B2FE67D
title: "Self-host lexer lex_next timeout blocks parser loader and module graph doctests"
area: selfhost
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-07
target: "stdlib/neplg2/core/syntax/lexer.nepl, stdlib/neplg2/core/syntax/parser/module_parser.nepl, stdlib/neplg2/core/module/loader.nepl, stdlib/neplg2/core/module/graph.nepl"
---

# ISS-20260506T193839798Z-SELF-HOST-LEXER-LEX-NEXT-TIMEOUT-BLO-6B2FE67D: Self-host lexer lex_next timeout blocks parser loader and module graph doctests

## 概要

On current main, even an empty lex_all_with_file_id smoke case times out at the default 60000ms wasm test budget. The graph, loader, and module_parser doctests also time out because they all enter lexer lex_next/lex_all. The stdlib_map timeout was separately reduced to a compile-time owner-leak diagnostic and then fixed, so the remaining graph timeout is rooted in lexer/static-check complexity rather than path mapping.

## 対象

- `stdlib/neplg2/core/syntax/lexer.nepl, stdlib/neplg2/core/syntax/parser/module_parser.nepl, stdlib/neplg2/core/module/loader.nepl, stdlib/neplg2/core/module/graph.nepl`

## 根拠

- `node nodesrc/tests.js -i tmp/probe_lex_empty.n.md --no-tree -o tmp/probe_lex_empty_after_remote_resource_fixes.json -j 1` on `5a8515ec` still timed out at compile phase after 60000ms.
- `NEPL_TEST_CASE_TIMEOUT_MS=240000 node nodesrc/tests.js -i tmp/probe_lex_empty.n.md --no-tree -o tmp/probe_lex_empty_long_timeout.json -j 1` also timed out before diagnostics.
- A local stdlib-only experiment split lexer internals, converted raw mode to enum state, removed token-wide predicate helper calls from `lex_all`, and rewrote the offside loop away from mutual recursion. The `lex_all_with_file_id` call still timed out, while import-only probes for the split submodules completed quickly.
- Compiler-side blocker issue `ISS-20260506T203121413Z-COMPILER-STATIC-CHECKER-TIMES-OUT-ON-5B942F4A` tracks the static/resource analysis timeout that prevents this self-host lexer issue from being closed by stdlib-only refactoring.
- 2026-05-07: `ISS-20260506T203121413Z-COMPILER-STATIC-CHECKER-TIMES-OUT-ON-5B942F4A` の compiler timeout は fixed。empty lexer smoke は timeout ではなく Resource owner diagnostics まで進むようになった。残る blocker は `ISS-20260506T224618064Z-SELF-HOST-LEXER-OWNER-FLOW-FAILS-AFT-23CB5BBE` で追跡する。

## 問題

On current main, even an empty lex_all_with_file_id smoke case times out at the default 60000ms wasm test budget. The graph, loader, and module_parser doctests also time out because they all enter lexer lex_next/lex_all. The stdlib_map timeout was separately reduced to a compile-time owner-leak diagnostic and then fixed, so the remaining graph timeout is rooted in lexer/static-check complexity rather than path mapping.

## 影響

Self-host parser/loader/import-graph doctests cannot provide CI signal, and graph work cannot verify import traversal while lex_next stays above the default per-case budget. The problem also hides whether graph DFS itself is correct.

## 修正方針

The compiler-side timeout tracked by `ISS-20260506T203121413Z-COMPILER-STATIC-CHECKER-TIMES-OUT-ON-5B942F4A` is fixed. Next, resolve `ISS-20260506T224618064Z-SELF-HOST-LEXER-OWNER-FLOW-FAILS-AFT-23CB5BBE` without weakening Resource owner diagnostics. After owner flow is correct, revisit lexer structure for maintainability: keep Copy-only token range classification, use enum state for raw modes, avoid temporary string owners while classifying identifiers/directives, and keep directive/keyword/token construction split so enum/match coverage remains explicit.

## 検証

Run node nodesrc/tests.js -i tmp/probe_lex_empty.n.md --no-tree -o tmp/probe_lex_empty_after_fix.json -j 1, then stdlib/neplg2/core/syntax/parser/module_parser.nepl, stdlib/neplg2/core/module/loader.nepl, and stdlib/neplg2/core/module/graph.nepl focused doctests under the default 60000ms timeout.

## 2026-05-07 closeout

`ISS-20260506T203121413Z-COMPILER-STATIC-CHECKER-TIMES-OUT-ON-5B942F4A`、`ISS-20260506T224618064Z-SELF-HOST-LEXER-OWNER-FLOW-FAILS-AFT-23CB5BBE`、`ISS-20260507T003424385Z-RESOURCE-OWNER-SUMMARY-DROPS-RAW-OWN-AE32128E` の修正後に、current `main` の compiler bundle を `trunk build` で更新して再検証した。

結果として、empty lexer smoke、module parser、module loader、module graph の各 doctest はすべて default 60000ms budget で passed になった。今回の issue の root blocker だった lex_next / lex_all timeout は解消済みと判断し、この issue は fixed とする。

検証:

- `trunk build`: passed
- `NEPL_TEST_CASE_TIMEOUT_MS=60000 node nodesrc/tests.js -i tmp/agent1_probe_lex_empty.n.md --no-tree --dist web/dist -o tmp/selfhost_lex_empty_timeout_closeout.json -j 1 --assert-io`: 1/1 passed
- `NEPL_TEST_CASE_TIMEOUT_MS=60000 node nodesrc/tests.js -i stdlib/neplg2/core/syntax/parser/module_parser.nepl --no-tree --dist web/dist -o tmp/selfhost_module_parser_timeout_closeout.json -j 1 --assert-io`: 1/1 passed
- `NEPL_TEST_CASE_TIMEOUT_MS=60000 node nodesrc/tests.js -i stdlib/neplg2/core/module/loader.nepl --no-tree --dist web/dist -o tmp/selfhost_loader_timeout_closeout.json -j 1 --assert-io`: 1/1 passed
- `NEPL_TEST_CASE_TIMEOUT_MS=60000 node nodesrc/tests.js -i stdlib/neplg2/core/module/graph.nepl --no-tree --dist web/dist -o tmp/selfhost_graph_timeout_closeout.json -j 1 --assert-io`: 1/1 passed
