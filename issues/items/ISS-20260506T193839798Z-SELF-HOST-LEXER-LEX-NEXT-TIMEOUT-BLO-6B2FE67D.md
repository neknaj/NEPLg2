---
id: ISS-20260506T193839798Z-SELF-HOST-LEXER-LEX-NEXT-TIMEOUT-BLO-6B2FE67D
title: "Self-host lexer lex_next timeout blocks parser loader and module graph doctests"
area: selfhost
status: investigating
resolved: false
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-14
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

## 2026-05-14 再発観測

`Vec.push` の owner-preserving failure payload 化後に current `web/dist` で module graph / lexer 系 focused doctest を再確認したところ、module graph は default 60000ms compile budget で 3/3 timeout した。lexer 側も full focused run が timeout / owner diagnostic を含んで完走せず、個別再実行では `resource.owner.maybe_leak` ではなく return value mismatch まで進むケースがある。

現時点では Vec.push API 変更が直接の根因とは判断しない。module graph は lexer / parser / loader を通るため、ResourceIR summary と selfhost lexer owner flow の静的検査コストを再監査し、timeout を単にテスト予算で隠さない。

再検証:

- `node nodesrc/tests.js -i tests/stdlib/neplg2_module_graph.n.md --no-tree -o tmp/agent1-vec-push-owner-error-neplg2-module-graph-final.json -j 1 --dist web/dist --assert-io`: total=3, errored=3, compile timeout
- `node nodesrc/tests.js -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/agent1-vec-push-owner-error-neplg2-lexer-final.json -j 1 --dist web/dist --assert-io`: partial run, timeout / owner diagnostics observed before later local error-path cleanup

## 2026-05-14 ResourceIR recursive variant owner fix checkpoint

`Result<Vec<T>, E>` を返す再帰関数で、callee の戻り値を caller がそのまま返す場合に、variant return target 側へ owner marker が既に存在すると元引数の owner leaf が未消費のまま残ることを reduced regression で再現した。これは `lex_all_loop` / `lex_line_start` のように `Result<Vec<SelfhostToken>, LexDiagnostic>` を相互再帰で受け渡す経路と同じ形で、`resource.owner.maybe_leak` の直接原因だった。

修正では pending variant owner return の materialize 時に、target に owner が既にあっても source が別 place でまだ transferable なら source を move-out するようにした。これにより、variant payload によって所有権が戻り値へ遅延移動される場合でも、元引数を caller 側に生かしたままにしない。

検証:

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_recursive_vec_result_err_does_not_keep_inactive_ok_owner -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_vec_push_error_owner_does_not_leak_through_result_err -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_preserves_branch_result_variant_owner_return -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_prefers_live_return_owner_over_moved_source_alias -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_does_not_reconsume_unconditional_variant_argument -- --nocapture`: passed
- `trunk build`: passed
- `NEPL_TEST_CASE_TIMEOUT_MS=60000 node nodesrc/tests.js -i tmp/agent1_probe_lex_empty.n.md --no-tree --dist web/dist -o tmp/agent1_probe_lex_empty_after_owner_variant_fix.json -j 1 --assert-io`: total=1, passed=1

残件:

- `tests/stdlib/neplg2_module_graph.n.md` は default 60000ms budget ではまだ compile timeout する。
- ただし `NEPL_TEST_CASE_TIMEOUT_MS=240000 node nodesrc/run_doctest.js -i tests/stdlib/neplg2_module_graph.n.md -n 1 --dist web/dist` は passed で、`compile_ms=65262`, `run_ms=17`。
- `-n 2` は passed で、`compile_ms=68010`, `run_ms=16`。
- `-n 3` は passed で、`compile_ms=68373`, `run_ms=14`。
- native stage timing では case 1 の `resource_static_check=36544ms`、主な内訳は `resource_initialized_raw_init_summaries=15828ms`, `resource_initialized_moves=18160ms`, `resource_owner_summaries=9421ms`, `resource_owner_obligations=9908ms`, `resource_effect_boundaries=6367ms`。

したがって、owner diagnostic blocker は解消したが、この issue はまだ close しない。残りは runtime や出力 wasm の問題ではなく、selfhost graph/import stack を含む大きめの入力で ResourceIR / initialized / owner / effect の静的検査が wasm runner の default 60s budget を超える性能問題として継続調査する。
