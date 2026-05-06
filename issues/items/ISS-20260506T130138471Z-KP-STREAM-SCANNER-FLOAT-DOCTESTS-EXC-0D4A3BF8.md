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

## 2026-05-06 fs/stdio owner 修正後の再確認

`ISS-20260506T130126516Z-RESOURCE-OWNER-SUMMARIES-REJECT-FS-A-7E58243F` の修正後、`tests/stdlib/kp.n.md` の doctest#5/#6 は timeout ではなく stdout を出して passed した。

確認結果:

- doctest#5: stdout `3.500000`, `-2.250000`, `100.000000`、実行時間は約 56.7 秒。
- doctest#6: stdout `1.250000`、実行時間は約 59.0 秒。

timeout symptom は解消したが、60 秒制限に近い実行時間は残っている。これは test budget 引き上げで隠すのではなく、float scanner / formatter / generated wasm の計算量と進捗性を確認する performance issue として open のまま継続する。

## 2026-05-06 string boundary 修正後の再確認

`ISS-20260506T135746003Z-STRING-ACCESS-SPLIT-LOSES-RAW-MEMORY-8C64A912` の修正後、`tests/stdlib/kp.n.md` の doctest#5/#6 は再び `wasm test case timeout after 60000ms` になった。

今回の実行は fs/stdio focused doctest と並行していたため単独実行での再測定が必要だが、`len__str` / scanner boundary の compile blocker は消えており、残る問題が runtime budget / algorithm / generated wasm 側に戻ったことは確認できた。この issue は引き続き open とする。

## 2026-05-06 unwrap_ok dealloc 修正後の再確認

`ISS-20260506T134653279Z-RESOURCE-OWNER-SUMMARY-MISSES-RAW-DE-007EB7EA` の修正後に `node nodesrc/tests.js -i tests/stdlib/kp.n.md -o output/kp_after_unwrap_ok_dealloc_summary.json --runner wasm --no-tree -j 1 --assert-io` を再実行した。

結果は total=7, passed=4, failed=1, errored=2 で、doctest#5/#6 はそれぞれ `wasm test case timeout after 60000ms` のまま残った。doctest#7 の owner leak が消えても float scanner/writer path の runtime budget 問題は独立して残るため、この performance issue は open のまま継続する。
