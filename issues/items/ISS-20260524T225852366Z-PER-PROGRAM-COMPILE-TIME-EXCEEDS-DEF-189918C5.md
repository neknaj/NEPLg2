---
id: ISS-20260524T225852366Z-PER-PROGRAM-COMPILE-TIME-EXCEEDS-DEF-189918C5
title: "Per-program compile time exceeds default budget after NEPLg2.1 static-check expansion"
area: core
status: open
resolved: false
priority: P1
type: performance
created: 2026-05-24
updated: 2026-05-25
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

## 問題

Current representative programs are dominated by compile phase cost. Even a tiny stdio print exceeds the default 60000ms case budget, and KP scanner programs need several minutes to compile while runtime remains around 10ms.

## 影響

Local and CI feedback can no longer distinguish semantic regressions from compile-time pressure. Performance regressions also hide whether NEPLg2.1 syntax migration and static-check fixes are functionally correct.

## 修正方針

Create a fixed per-program benchmark corpus, keep compile_ms and run_ms evidence in issues, profile compiler stage timing, and reduce the dominant static-check/import-graph cost at the root. Do not solve this by globally raising timeouts or deleting coverage.

まず、代表 program を次の階層に分けて固定する。

- core only: stdlib import を含まない `#target core` の最小 return case。
- stdio only: `std/stdio` だけで 1 行出力する case。
- stream writer: `std/streamio` の writer を explicit value chain で使う case。
- stream scanner: `StreamScanner` read と stdio / writer output を組み合わせる case。
- KP workload: prefix sum, float scanner/writer, kpsearch のように stdlib graph と Resource IR summary が大きくなる case。

その上で、`NEPL_COMPILE_STAGE_TIMING=1` の native timing と wasm doctest の `timing.compile_ms` を対応付ける。候補は Resource IR summary propagation、stdlib import graph の解析範囲、diagnostic materialization、型推論候補展開、WAT/comment 補助生成である。timeout 延長、coverage 削除、旧記法のままの失敗を性能改善として扱うことはしない。

## 検証

- `trunk build`
- `node nodesrc/run_doctest.js -i tmp/agent_perf_cases_20260525.n.md -n 1 --dist web/dist`
- `node nodesrc/run_doctest.js -i tmp/agent_perf_cases_20260525.n.md -n 2 --dist web/dist`
- `node nodesrc/run_doctest.js -i tmp/agent_perf_cases_20260525.n.md -n 3 --dist web/dist`
- `node nodesrc/run_doctest.js -i tmp/agent_perf_cases_20260525.n.md -n 4 --dist web/dist`
- `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 1 --dist web/dist`
- `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 3 --dist web/dist`
- `node nodesrc/tests.js -i tests/stdlib/kp.n.md --no-tree -o tmp/agent_perf_kp_20260525.json -j 1 --assert-io --dist web/dist`
