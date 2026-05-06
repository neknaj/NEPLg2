---
id: ISS-20260506T130138471Z-KP-STREAM-SCANNER-FLOAT-DOCTESTS-EXC-0D4A3BF8
title: "KP stream scanner float doctests exceed wasm runtime budget"
area: stdlib
status: open
resolved: false
priority: P1
type: performance
created: 2026-05-06
updated: 2026-05-06
target: "tests/stdlib/kp.n.md, stdlib/std/streamio/scanner/number.nepl, stdlib/std/streamio/writer, stdlib/core/float.nepl, nodesrc/tests.js"
---

# ISS-20260506T130138471Z-KP-STREAM-SCANNER-FLOAT-DOCTESTS-EXC-0D4A3BF8: KP stream scanner float doctests exceed wasm runtime budget

## 概要

After compile-time Stage5 effect blockers are removed, tests/stdlib/kp.n.md doctest#5 and doctest#6 no longer report effect diagnostics but exceed the 60000ms wasm doctest budget. Focused run_doctest for #5/#6 also takes about 61-64s and produces no stdout, so the issue is in runtime behavior or generated wasm performance for f64/f32 scanner-to-writer paths, not a remaining raw-memory compile diagnostic.

## 対象

- `tests/stdlib/kp.n.md, stdlib/std/streamio/scanner/number.nepl, stdlib/std/streamio/writer, stdlib/core/float.nepl, nodesrc/tests.js`

## 根拠

- `node nodesrc/tests.js -i tests/stdlib/kp.n.md -o output/kp_stage5_raw_boundary.json --runner wasm --no-tree -j 1 --assert-io` では doctest#5 と doctest#6 が `wasm test case timeout after 60000ms` になった。
- `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 5 --dist web/dist` は約 61 秒で stdout mismatch、actual stdout empty になった。
- `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 6 --dist web/dist` は約 64 秒で stdout mismatch、actual stdout empty になった。
- 同じ file の integer scanner cases は compile-time owner / range issue で止まるか、別経路で進むため、現時点で timeout は f64/f32 scanner read + writer formatting path に集中している。
- compile diagnostic は出ていないため、Stage 5 raw memory boundary の残りではなく runtime behavior / generated wasm / float parser・formatter の計算量問題として扱う。

## 問題

After compile-time Stage5 effect blockers are removed, tests/stdlib/kp.n.md doctest#5 and doctest#6 no longer report effect diagnostics but exceed the 60000ms wasm doctest budget. Focused run_doctest for #5/#6 also takes about 61-64s and produces no stdout, so the issue is in runtime behavior or generated wasm performance for f64/f32 scanner-to-writer paths, not a remaining raw-memory compile diagnostic.

## 影響

The kp regression file cannot become a stable CI signal. The timeout may indicate inefficient float parsing/formatting, generated wasm that fails to make progress, or an overly large doctest, so it must be profiled rather than hidden by increasing timeouts.

## 修正方針

Profile the f64/f32 scanner read and writer formatting path, compare integer scanner cases, and determine whether the cause is algorithmic complexity, generated wasm/codegen behavior, or test scope. Fix the root cause or split tests only if profiling shows the program work is inherently too large.

## 検証

- `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 5 --dist web/dist`
- `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 6 --dist web/dist`
- `node nodesrc/tests.js -i tests/stdlib/kp.n.md -o output/kp_float_runtime.json --runner wasm --no-tree -j 1 --assert-io`
