---
id: ISS-20260524T225852366Z-PER-PROGRAM-COMPILE-TIME-EXCEEDS-DEF-189918C5
title: "Per-program compile time exceeds default budget after NEPLg2.1 static-check expansion"
area: core
status: open
resolved: false
priority: P1
type: performance
created: 2026-05-24
updated: 2026-05-28
target: "nepl-core static check; nodesrc/tests.js; nodesrc/run_doctest.js; tests/stdlib/kp.n.md; stdlib/std/streamio; stdlib/std/stdio"
---

# ISS-20260524T225852366Z-PER-PROGRAM-COMPILE-TIME-EXCEEDS-DEF-189918C5: Per-program compile time exceeds default budget after NEPLg2.1 static-check expansion

## 概要

Current representative programs are dominated by compile phase cost. Even a tiny stdio print exceeds the default 60000ms case budget, and KP scanner programs need several minutes to compile while runtime remains around 10ms.

## 対象

- `nepl-core static check; nodesrc/tests.js; nodesrc/run_doctest.js; tests/stdlib/kp.n.md; stdlib/std/streamio; stdlib/std/stdio`

## 根拠

- 2026-05-25 に `trunk build` を通し、current branch の `web/dist` (`nepl-web-a37a58fd9167e6db_bg.wasm`) を使って測定した。
- `tmp/agent_perf_cases_20260525.n.md::doctest#1` は `#target core` の `i32` return だけの最小 case で、`compile_ms=15629`, `run_ms=8` だった。
- `tmp/agent_perf_cases_20260525.n.md::doctest#2` は `std/stdio` の `println_i32 42` だけの case で、`compile_ms=84980`, `run_ms=7` だった。最小 stdio program が default 60000ms budget を超える。
- `tmp/agent_perf_cases_20260525.n.md::doctest#3` は explicit `StreamWriter` で `i32` を 1 行出す case で、`compile_ms=62197`, `run_ms=9` だった。
- `tmp/agent_perf_cases_20260525.n.md::doctest#4` は explicit `StreamWriter` で `f32` を 1 行出す case で、`compile_ms=62278`, `run_ms=9` だった。float formatting 自体より、stream writer / imported stdlib graph の compile phase が支配的に見える。
- `tests/stdlib/kp.n.md::doctest#1` は `compile_ms=193644`, `run_ms=10`、`doctest#3` は `compile_ms=183741`, `run_ms=9` だった。どちらも実行時間ではなく compile phase が支配的である。
- Rust integration の `cargo test -p nepl-core --test kp -- --nocapture` は 15/15 passed だが、全体で 336.74s かかった。`kpwrite_f64_stdout_no_input` / `kpwrite_f32_stdout_no_input` のような scanner なし writer-only smoke でも各 case は 50 秒台で、compiler/static-check cost が支配的である。
- `cargo test -p nepl-core --test resource_ir resource_ir_vec_filter_drop_payload_uses_transform_range_certificate -- --exact --nocapture` は 2026-05-25 の修正後に pass したが、再検証でも test body 実行だけで 434.45s かかった。compile は 12.28s で、対象 program の static-check / Resource IR summary cost が支配的である。
- `cargo test -p nepl-core --test resource_ir resource_ir_initialized_check_vec_drop_push_free_closes_stdlib_lifecycle -- --exact --nocapture` は 300s timeout した。単一 Vec lifecycle integration でも default 作業 budget を超えるため、Vec collection summary 系も固定 benchmark corpus に含める必要がある。
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_source_drop_traversal_storage_release -- --exact --nocapture` は pass したが、test body 実行に 129.60s かかった。collection-slot source の小さめの回帰でも Resource IR cost が無視できない。
- `node nodesrc/tests.js -i tests/stdlib/kp.n.md --no-tree -o tmp/agent_perf_kp_20260525.json -j 1 --assert-io --dist web/dist` は default 60000ms budget で `total=7, passed=0, failed=4, errored=3` だった。`doctest#1/#3/#7` は compile timeout、`doctest#2/#4/#5/#6` は旧 pipe 形が `type.pipe.invalid` などで compile fail したため、性能問題と NEPLg2.1 syntax migration 残件を分けて扱う必要がある。
- 既存の `ISS-20260506T130138471Z-KP-STREAM-SCANNER-FLOAT-DOCTESTS-EXC-0D4A3BF8` は fixed 時点で focused suite 内 `doctest#1 compile_ms=14200ms`, `doctest#3 compile_ms=11800ms`, float cases も 20 秒台まで下がっていた。現在の 180-190 秒台はその issue の単純な継続ではなく、現 branch の compile-time regression として扱う。
- 2026-05-25 追加確認として `node --experimental-strip-types repo_metrics.ts --json tmp\repo_metrics_20260525_perf_checkpoint.json` を実行し、current working tree が 3,255 files / 545,392 lines / 3,364 test cases であることを確認した。
- `node nodesrc/run_doctest.js -i tmp/agent_perf_cases_20260525.n.md -n 2 --dist web/dist` は 184s local command timeout になった。最小 stdio case は wasm doctest 経路でも既に通常確認に使いにくい。
- rebuilt `target\debug\nepl-cli.exe --check -i tmp\perf_tiny_stdio_print_i32.neplg2 --target std` は 120s / 240s の local command timeout になった。古い binary の測定値と混同せず、current branch を rebuild した native CLI でも stdio 最小 case が長時間化しているものとして扱う。
- `NEPL_COMPILE_STAGE_TIMING=1 cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_source_drop_traversal_storage_release -- --exact --nocapture` は pass し、test body 31.25s だった。内訳は `resource_initialized_raw_alias_summaries=49ms`, `resource_initialized_i32_scalar_summaries=1560ms`, `resource_initialized_raw_init_summaries=15291ms`, `resource_initialized_collection_slot_summaries=7297ms`, `resource_initialized_function_checks=6717ms` で、raw init summary / collection slot summary / function check が支配的である。
- 試行として raw initialization summary を RawMemory / host effect / indirect call / relevant callee だけに絞る relevance filter を検討したが、`resource_ir_collection_slot_source_drop_traversal_storage_release` で `region_size` / `region_in_bounds` / `region_ptr` / `region_ptr_at` の reference parameter deref が `CellUnavailable` になった。このため、単純な direct RawMemory pruning は不正であり、reference parameter から導かれる initialized-cell 前提と summary relevance を同時に設計する必要がある。

## 問題

Current representative programs are dominated by compile phase cost. Even a tiny stdio print exceeds the default 60000ms case budget, and KP scanner programs need several minutes to compile while runtime remains around 10ms.

## 影響

Local and CI feedback can no longer distinguish semantic regressions from compile-time pressure. Performance regressions also hide whether NEPLg2.1 syntax migration and static-check fixes are functionally correct.

## 修正方針

Create a fixed per-program benchmark corpus, keep compile_ms and run_ms evidence in issues, profile compiler stage timing, and reduce the dominant static-check/import-graph cost at the root. Do not solve this by globally raising timeouts or deleting coverage.

2026-05-27 の修正で、最小 program / small aggregate の root cause は次の 3 点に分解された。

- Resource IR が entry から到達しない stdlib functions まで summary 固定点へ入れていた。
- default prelude の `Copy` capability import が `core/mem` allocator graph まで引いていた。
- Node runner が release artifact より古くない stdlib に対しても full FS stdlib VFS を毎回 WASM API へ渡していた。

対応として、Resource IR 前の entry reachability pruning、`core/traits/copy` と `core/mem/types` の依存境界整理、Node/WASM の bundled stdlib freshness 判定を入れた。設計詳細は [NEPLg2.1 compiler performance / cache design 2026-05-27](../../doc/neplg2/compiler_performance_cache_design.md) に固定した。

この issue は、stdlib-heavy / KP / collection lifecycle の representative corpus を 0.5 秒未満へ戻す親 issue として open のまま維持する。微小変更 10ms 未満のための CompilerSession / prechecked stdlib artifact は [ISS-20260527T050120000Z-COMPILER-SESSION-STDLIB-PRECHECK-CACHE-A71E4C92](./ISS-20260527T050120000Z-COMPILER-SESSION-STDLIB-PRECHECK-CACHE-A71E4C92.md) に分離した。

2026-05-27 checkpoint の測定:

- native minimal check: elapsed 160ms、`resource_typecheck=5ms`、`resource_static_check=1ms`。
- native aggregate check: elapsed 166ms、`resource_typecheck=6ms`、`resource_static_check=1ms`。
- release WASM minimal cold: `compile_ms=231`, `total_ms=257`, `stdlib_vfs_mode=bundled`。
- release WASM minimal warm: `compile_ms=5`。
- release WASM aggregate warm: `compile_ms=22`。

2026-05-28 checkpoint の測定:

- native release RPN check: `resource_static_check=9202ms`、`resource_initialized_i32_scalar_summaries=2012ms`、`resource_initialized_raw_init_summaries=2520ms`、`resource_initialized_function_checks=3730ms`。
- release WASM RPN same-session first compile: `compile_ms=8976`、`prewarm_ms=193`、`wasm_call_ms=8783`。
- release WASM RPN same-session second compile: `compile_ms=1`、`wasm_call_ms=0`、`compiled_output_cache_hits=1`。
- Web playground compile timeout の直接原因として、source-directed prewarm 後にまだ消費していない dependency aggregate public surface hash を同期計算していたことを確認した。RPN ではこの追加 query が private implementation graph を広く歩き、wasm doctest が compile phase で 120 秒 timeout したため、Web prewarm hot path から外した。
- Resource IR query pruning checkpoint では、local transform-range certificate を `CollectionSlotTransformRange` 消費関数だけで構築するようにし、i32 scalar return leaf relation 収集では `I32ConditionQueryContext` を leaf pair ごとではなく relation 収集全体で共有した。
- 同 checkpoint の native release RPN stage-only 測定は `resource_static_check=8389ms`、`resource_initialized_i32_scalar_summaries=1372ms`、`resource_initialized_raw_init_summaries=2613ms`、`resource_initialized_function_checks=3470ms`。i32 scalar summary は軽くなったが、raw init summary / function check がまだ支配的である。
- `I32ConditionQueryContext` 全体の `BTreeMap` 化と loop initialized range body guard は実測で悪化したため採用しなかった。次の根本対応は typed public signature table を invalidation 境界にした Resource IR summary cache である。

RPN では同一入力の再compileは 10ms 未満になったが、初回 compile はまだ 0.5 秒未満から遠い。次の根本対応は raw init summary / function check の path-sensitive exploration を function hash と dependency aggregate public surface hash で再利用する Resource IR summary cache である。

まず、代表 program を次の階層に分けて固定する。

- core only: stdlib import を含まない `#target core` の最小 return case。
- stdio only: `std/stdio` だけで 1 行出力する case。
- stream writer: `std/streamio` の writer を explicit value chain で使う case。
- stream scanner: `StreamScanner` read と stdio / writer output を組み合わせる case。
- KP workload: prefix sum, float scanner/writer, kpsearch のように stdlib graph と Resource IR summary が大きくなる case。

その上で、`NEPL_COMPILE_STAGE_TIMING=1` の native timing と wasm doctest の `timing.compile_ms` を対応付ける。候補は Resource IR summary propagation、stdlib import graph の解析範囲、diagnostic materialization、型推論候補展開、WAT/comment 補助生成である。timeout 延長、coverage 削除、旧記法のままの失敗を性能改善として扱うことはしない。

2026-05-25 の追加調査により、raw initialization summary の単純な relevance pruning は却下した。次の候補は、summary builder が reference parameter / raw address alias / initialized-cell seed をどの関数で必要とするかを明示的な fact category に分け、その category ごとに worklist を分けることである。`RawMemory` を持つ関数だけを残すのではなく、reference deref の可用性を証明する lightweight summary と raw byte/cell mutation summary を分ける必要がある。

## 検証

- `trunk build`
- `node nodesrc/run_doctest.js -i tmp/agent_perf_cases_20260525.n.md -n 1 --dist web/dist`
- `node nodesrc/run_doctest.js -i tmp/agent_perf_cases_20260525.n.md -n 2 --dist web/dist`
- `node nodesrc/run_doctest.js -i tmp/agent_perf_cases_20260525.n.md -n 3 --dist web/dist`
- `node nodesrc/run_doctest.js -i tmp/agent_perf_cases_20260525.n.md -n 4 --dist web/dist`
- `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 1 --dist web/dist`
- `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 3 --dist web/dist`
- `node nodesrc/tests.js -i tests/stdlib/kp.n.md --no-tree -o tmp/agent_perf_kp_20260525.json -j 1 --assert-io --dist web/dist`
