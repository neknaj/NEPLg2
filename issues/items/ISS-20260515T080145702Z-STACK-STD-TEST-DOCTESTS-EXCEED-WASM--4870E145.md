---
id: ISS-20260515T080145702Z-STACK-STD-TEST-DOCTESTS-EXCEED-WASM--4870E145
title: "Stack std/test doctests exceed wasm compiler timeout after static-check expansion"
area: core
status: open
resolved: false
priority: P2
type: performance
created: 2026-05-15
updated: 2026-05-15
target: "nepl-core/src/resource, nodesrc/tests.js, stdlib/tests/stack.n.md, tests/stdlib/stack_collections.n.md"
---

# ISS-20260515T080145702Z-STACK-STD-TEST-DOCTESTS-EXCEED-WASM--4870E145: Stack std/test doctests exceed wasm compiler timeout after static-check expansion

## 概要

Focused verification of stdlib/tests/stack.n.md and tests/stdlib/stack_collections.n.md now times out in compile phase for every std/test-based Stack doctest under the default 60000ms wasm case budget; individual run_doctest attempts for stack_pop_top_keeps_stack also did not finish within 180000ms. A minimal StackPop accessor smoke without std/test compiles and runs in about 10s, and examples bf/rpn/rpn_legacy pass, so the timeout appears tied to compiler/static-check cost for the full std/test doctest shape rather than generated wasm runtime.

## 対象

- `nepl-core/src/resource, nodesrc/tests.js, stdlib/tests/stack.n.md, tests/stdlib/stack_collections.n.md`

## 根拠

- `node nodesrc/tests.js -i stdlib/tests/stack.n.md -i tests/stdlib/stack_collections.n.md --no-tree -o tmp/agent1-stack-pop-accessors-doctests.json -j 1 --dist web/dist --assert-io` は外側 command が 604 秒で timeout した。partial JSON は completed_results=10/18、すべて compile phase の `wasm test case timeout after 60000ms`。
- `node nodesrc/run_doctest.js -i stdlib/tests/stack.n.md -n 9 --assert-io --dist web/dist` と `node nodesrc/run_doctest.js -i tests/stdlib/stack_collections.n.md -n 8 --assert-io --dist web/dist` は、それぞれ外側 command の 184 秒 timeout まで完了しなかった。
- 一方で、`StackPop` accessor だけを使う最小 smoke を `nodesrc/run_test.js` に渡すと compile_ms 約 10120ms で実行まで成功した。
- `examples/bf.nepl` / `examples/rpn.nepl` / `examples/rpn_legacy.nepl` は同じ accessor API を使って total=5, passed=5 で通過したため、生成 wasm runtime の hang ではなく、`std/test` doctest shape と現在の静的検査の組み合わせによる compile-time cost と推定する。

## 問題

Focused verification of stdlib/tests/stack.n.md and tests/stdlib/stack_collections.n.md now times out in compile phase for every std/test-based Stack doctest under the default 60000ms wasm case budget; individual run_doctest attempts for stack_pop_top_keeps_stack also did not finish within 180000ms. A minimal StackPop accessor smoke without std/test compiles and runs in about 10s, and examples bf/rpn/rpn_legacy pass, so the timeout appears tied to compiler/static-check cost for the full std/test doctest shape rather than generated wasm runtime.

## 影響

Agents cannot use the canonical Stack doctest suite as a short local regression gate, and a real static-check complexity regression may be hidden behind command-level timeouts. Raising timeouts would only mask the problem; the compiler should prove owner/effect properties for these doctests within the normal budget.

## 修正方針

Profile native and wasm compile stages for a representative stack_pop_top_keeps_stack doctest with NEPL_COMPILE_STAGE_TIMING, identify whether type, owner, lifetime, or effect/resource summaries dominate, and fix the compiler algorithm or summary invalidation path instead of weakening static checks or removing std/test assertions.

## 検証

Run the two Stack doctest files with nodesrc/tests.js under the default timeout and confirm all Stack doctests pass; keep the minimal StackPop smoke and examples passing.
