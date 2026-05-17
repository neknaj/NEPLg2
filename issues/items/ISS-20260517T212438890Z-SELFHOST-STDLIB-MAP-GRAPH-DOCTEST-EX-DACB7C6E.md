---
id: ISS-20260517T212438890Z-SELFHOST-STDLIB-MAP-GRAPH-DOCTEST-EX-DACB7C6E
title: "selfhost stdlib_map graph doctest exceeds default compile timeout"
area: CORE
status: open
resolved: false
priority: P1
type: performance
created: 2026-05-17
updated: 2026-05-17
target: "tests/stdlib/neplg2_stdlib_map.n.md, nepl-core/src/resource"
---

# ISS-20260517T212438890Z-SELFHOST-STDLIB-MAP-GRAPH-DOCTEST-EX-DACB7C6E: selfhost stdlib_map graph doctest exceeds default compile timeout

## 概要

tests/stdlib/neplg2_stdlib_map.n.md::doctest#2 passes when NEPL_TEST_CASE_TIMEOUT_MS is raised to 300000, but under the default 60000ms wasm case budget it times out in compile phase. The focused long run measured compile_ms about 74772 and run_ms about 29, so the bottleneck is compiler/static-check cost for the selfhost graph/std-test shape, not generated wasm execution.

## 対象

- `tests/stdlib/neplg2_stdlib_map.n.md, nepl-core/src/resource`

## 根拠

- `node nodesrc/tests.js -i tests\stdlib\neplg2_stdlib_map.n.md --no-tree -o tmp\agent1-neplg2-stdlib-map-owner-summary.json -j 1 --dist web\dist --assert-io` は total=3, passed=2, errored=1。`doctest#2` だけが compile phase で `wasm test case timeout after 60000ms` になった。
- `$env:NEPL_TEST_CASE_TIMEOUT_MS='300000'; node nodesrc\run_doctest.js -i tests\stdlib\neplg2_stdlib_map.n.md -n 2 --assert-io --dist web\dist` は pass し、`compile_ms=74772`, `run_ms=29`, `total_ms=74850` だった。
- `doctest#1` と `doctest#3` は同じ dist で compile 約22秒、run 約20ms以下で pass しているため、今回の ResourceIR owner diagnostic 解消とは別に、graph/VFS/std-test fixture の compile-time cost が default timeout を超えている。

## 問題

tests/stdlib/neplg2_stdlib_map.n.md::doctest#2 passes when NEPL_TEST_CASE_TIMEOUT_MS is raised to 300000, but under the default 60000ms wasm case budget it times out in compile phase. The focused long run measured compile_ms about 74772 and run_ms about 29, so the bottleneck is compiler/static-check cost for the selfhost graph/std-test shape, not generated wasm execution.

## 影響

The stdlib_map graph regression cannot be used as a normal local/CI focused gate under the default timeout, and static-check complexity regressions can be mistaken for runtime behavior unless this path is optimized or split with a principled test strategy.

## 修正方針

Investigate compile-stage timing for the graph/VFS/std-test fixture and reduce ResourceIR/type/effect summary work from the compiler side. Do not solve by only raising timeout. If the test is inherently too broad, split it into smaller doctests while preserving stdout assertions.

## 検証

Run tests/stdlib/neplg2_stdlib_map.n.md::doctest#2 under the default 60000ms timeout with --assert-io and confirm it passes; keep doctest#1/#3 and ResourceIR owner summary regressions passing.
