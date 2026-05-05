---
id: ISS-20260505T033328864Z-BYTEBUILDER-LEB128-DOCTEST-EXCEEDS-6-8F319A4D
title: "ByteBuilder LEB128 doctest exceeds 60s wasm case timeout"
area: tests
status: open
resolved: false
priority: P2
type: performance
created: 2026-05-05
updated: 2026-05-05
target: tests/stdlib/byte_builder.n.md
---

# ISS-20260505T033328864Z-BYTEBUILDER-LEB128-DOCTEST-EXCEEDS-6-8F319A4D: ByteBuilder LEB128 doctest exceeds 60s wasm case timeout

## 概要

After the Resource IR owner summary fix, tests/stdlib/byte_builder.n.md no longer reports builder owner leaks, but doctest#2 times out under the default 60000ms wasm case timeout. A focused run with NEPL_TEST_CASE_TIMEOUT_MS=180000 passes and reports duration_ms=153831, while doctest#1 and doctest#3 pass at roughly 54s and 43s. This suggests the case is dominated by compile/static-check cost or fixture granularity, not a functional byte_builder failure.

## 対象

- `tests/stdlib/byte_builder.n.md`

## 根拠

- `node nodesrc/tests.js -i tests/stdlib/byte_builder.n.md --no-tree -o tmp/byte-builder-after-owner-value-source.json -j 1 --dist web/dist`: total=3, passed=2, errored=1。
- `doctest#2` は `wasm test case timeout after 60000ms`。同じ run で `doctest#1` は duration_ms=54331、`doctest#3` は duration_ms=42775 で pass しており、owner leak は top issue に出ていない。
- `NEPL_TEST_CASE_TIMEOUT_MS=180000 node nodesrc/run_doctest.js -i tests/stdlib/byte_builder.n.md -n 2 --dist web/dist`: pass, duration_ms=153831。
- `doctest#2` は known vector `624485 -> E5 8E 26` の小さい runtime 検査であり、実行アルゴリズム自体が 154 秒必要な内容ではない。現時点の有力仮説は stdlib 込み compile / Resource IR static-check / std/test aggregation の負荷、または fixture 粒度の過大化である。

## 問題

After the Resource IR owner summary fix, tests/stdlib/byte_builder.n.md no longer reports builder owner leaks, but doctest#2 times out under the default 60000ms wasm case timeout. A focused run with NEPL_TEST_CASE_TIMEOUT_MS=180000 passes and reports duration_ms=153831, while doctest#1 and doctest#3 pass at roughly 54s and 43s. This suggests the case is dominated by compile/static-check cost or fixture granularity, not a functional byte_builder failure.

## 影響

The ByteBuilder regression file cannot be used as a clean focused signal under the default runner budget. Future owner or emitter regressions can be hidden behind timeout noise, and simply increasing the timeout would mask possible compile-time complexity regressions.

## 修正方針

Profile doctest#2 with compile-only and runtime timing, inspect Resource IR/static-check complexity for byte_builder_push_leb_u32 and std/test aggregation, then either reduce compiler complexity or split the fixture so each case stays comfortably below the default timeout without weakening checks.

## 検証

Run tests/stdlib/byte_builder.n.md with the default timeout and require all 3 doctests to pass. Also record compile-only vs runtime timings so the resolution is tied to the actual cause rather than a timeout increase.
