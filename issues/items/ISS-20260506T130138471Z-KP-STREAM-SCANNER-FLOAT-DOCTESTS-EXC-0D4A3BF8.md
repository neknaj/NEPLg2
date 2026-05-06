---
id: ISS-20260506T130138471Z-KP-STREAM-SCANNER-FLOAT-DOCTESTS-EXC-0D4A3BF8
title: "KP stream scanner doctests exceeded compiler/runtime budget"
area: core
status: fixed
resolved: true
priority: P1
type: performance
created: 2026-05-06
updated: 2026-05-07
target: "nepl-core/src/resource/*summary*.rs, nepl-core/src/compiler.rs, nepl-cli/src/main.rs, nepl-web/src/lib.rs, tests/stdlib/kp.n.md"
---

# ISS-20260506T130138471Z-KP-STREAM-SCANNER-FLOAT-DOCTESTS-EXC-0D4A3BF8: KP stream scanner doctests exceed compiler/runtime budget

## 概要

After compile-time Stage5 effect blockers are removed, tests/stdlib/kp.n.md doctests remain close to or above the wasm doctest budget. Earlier reports focused on f64/f32 scanner-to-writer runtime behavior, but current phase timing shows even the small integer scanner case is dominated by compiler wasm compile time. This issue now tracks the KP stream scanner regression file as a compiler/runtime budget problem until phase-level profiling identifies and fixes the actual hot path.

## 対象

- `tests/stdlib/kp.n.md, stdlib/std/streamio/scanner/number.nepl, stdlib/std/streamio/writer, stdlib/core/float.nepl, nodesrc/tests.js`

## 根拠

- `node nodesrc/tests.js -i tests/stdlib/kp.n.md -o output/kp_stage5_raw_boundary.json --runner wasm --no-tree -j 1 --assert-io` では doctest#5 と doctest#6 が `wasm test case timeout after 60000ms` になった。
- `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 5 --dist web/dist` は約 61 秒で stdout mismatch、actual stdout empty になった。
- `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 6 --dist web/dist` は約 64 秒で stdout mismatch、actual stdout empty になった。
- 同じ file の integer scanner cases は compile-time owner / range issue で止まるか、別経路で進むため、現時点で timeout は f64/f32 scanner read + writer formatting path に集中している。
- compile diagnostic は出ていないため、Stage 5 raw memory boundary の残りではなく runtime behavior / generated wasm / float parser・formatter の計算量問題として扱う。

## 問題

After compile-time Stage5 effect blockers are removed, tests/stdlib/kp.n.md doctests remain close to or above the wasm doctest budget. Earlier reports focused on f64/f32 scanner-to-writer runtime behavior, but current phase timing shows even the small integer scanner case is dominated by compiler wasm compile time. This issue now tracks the KP stream scanner regression file as a compiler/runtime budget problem until phase-level profiling identifies and fixes the actual hot path.

## 影響

The kp regression file cannot become a stable CI signal. The timeout may indicate inefficient float parsing/formatting, generated wasm that fails to make progress, or an overly large doctest, so it must be profiled rather than hidden by increasing timeouts.

## 修正方針

Use phase timing from `ISS-20260506T183933332Z-DOCTEST-TIMEOUT-REPORTS-LACK-COMPILE-9FFADD53` to profile integer and float KP doctests separately. First identify whether the hot path is compiler wasm compile time, Resource IR/static check algorithmic complexity, generated wasm/codegen behavior, or actual scanner/writer runtime work. Fix the root cause; split tests only if profiling proves the program work itself is inherently too large.

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

## 2026-05-06 phase timing 追加後の再確認

`ISS-20260506T183933332Z-DOCTEST-TIMEOUT-REPORTS-LACK-COMPILE-9FFADD53` の調査で、`tests/stdlib/kp.n.md::doctest#1` は passed するが `timing.compile_ms=47510`, `timing.run_ms=14`, `timing.total_ms=47565` だった。これは小さい i32 scanner case でも runtime ではなく compiler wasm compile phase が支配的であることを示す。

さらに `NEPL_TEST_CASE_TIMEOUT_MS=2000 node nodesrc/tests.js -i tests/stdlib/kp.n.md --no-tree -o tmp/agent1-kp-timeout-phase.json -j 1 --assert-io` では全 7 case が意図した短時間 timeout になり、top issue の `phase` は `compile`、JSON の `timeout.last_phase` も `compile` だった。

したがって、この issue の旧説明にあった「float runtime path に集中」という前提は現在の main では不十分である。次の修正では compiler wasm compile phase をさらに Rust compiler phase 別に分解し、Resource IR / monomorphize / codegen / stdlib import graph のどれが支配的かを確認する。

## 2026-05-07 Resource summary worklist 修正後の解決確認

原因は stdlib / KP runtime ではなく、Resource IR static check の関数 summary 計算量だった。`resource_initialized_raw_init_summaries` と `resource_owner_summaries` が、関数ごとの依存関係を見ずに全関数を全反復で再計算していたため、KP の小さい doctest でも stdlib import graph 全体の summary 再計算が compile phase を支配していた。

対応:

- `initialized` / `owner` summary builder を in-place fixed point 更新へ変更した。
- Resource IR 関数 summary の direct call / function value / nested branch / loop / match dependency graph を作り、変化した callee summary に依存する caller だけを worklist で再計算するようにした。
- wasm-only compile API では WAT コメント補助情報を生成せず、WAT 出力を要求する経路だけ `CompilationArtifactOptions.include_wat_comments` を有効にするようにした。
- `NEPL_COMPILE_STAGE_TIMING=1` による host-only stage timing を追加し、wasm32 runtime では `Instant::now()` が実行されない cfg にした。
- `summary_dependency` の単体回帰で direct call、function value、nested branch、self recursion の dependent graph を固定した。

計測:

- 修正前: `tests/stdlib/kp.n.md::doctest#1` は `compile_ms=47510`, `run_ms=14`。
- in-place summary 後: doctest#1 は `compile_ms=31498`。
- worklist summary 後: doctest#1 は単体 `compile_ms=17580`、focused suite 内では `compile_ms=16364`。
- native stage timing では `resource_static_check` が約 15.9 秒から約 6.7 秒へ低下した。内訳は `resource_initialized_raw_init_summaries=3956ms`, `resource_owner_summaries=1103ms`。
- `tests/stdlib/kp.n.md` focused suite は total=7, passed=7。最終 refactor 後の focused suite 内 compile time は doctest#1 14.2s、#2 10.0s、#3 11.8s、#4 11.2s、#5 18.1s、#6 20.3s、#7 6.3s。

現在の 60 秒 timeout budget 超過は解消したため、この issue は fixed とする。各 case の compile がまだ数秒から 20 秒台である点は残る改善余地だが、今回の「KP doctest が budget を超える」主症状とは分ける。
