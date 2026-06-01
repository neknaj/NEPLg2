---
id: ISS-20260524T225852366Z-PER-PROGRAM-COMPILE-TIME-EXCEEDS-DEF-189918C5
title: "Per-program compile time exceeds default budget after NEPLg2.1 static-check expansion"
area: core
status: open
resolved: false
priority: P1
type: performance
created: 2026-05-24
updated: 2026-06-02
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
- merged literal fast path checkpoint では、path-sensitive replay 後に merged state だけで実行してよい `ResourceOp::Expr` を fresh temporary の `LiteralI32` / `LayoutSizeOf` に限定して追加した。local や projection 付き output は path ごとの alias / scalar fact を壊し得るため対象外にしている。
- 同 checkpoint の native release RPN stage-only 測定は `resource_static_check=8033ms`、`resource_initialized_i32_scalar_summaries=1309ms`、`resource_initialized_raw_init_summaries=2509ms`、`resource_initialized_function_checks=3317ms`。初回 compile はまだ 0.5 秒未満ではないが、function check の path-sensitive exploration をさらに削減できた。
- typed public signature checkpoint では、typecheck 成功時に arena 非依存の `TypedPublicSignatureTable` を返すようにした。public callable / struct / enum / trait / impl header の stable text と hash だけを持ち、`TypeId`、`Span`、typed HIR、Resource IR は保存しない。
- 同 checkpoint の regression では、public function の body-only edit で hash が不変、public callable return type edit で hash が変化することを確認した。次はこの hash を dependency aggregate public surface hash と組み合わせて Resource IR summary cache の invalidation key に使う。
- typed public signature pipeline staging checkpoint では、`TypedPublicSignatureTable` を `PreparedProgram` まで運ぶようにした。cache value の再利用はまだ行わず、後段で Resource IR summary cache key を構築できるようにするための入力だけを渡している。
- variant-param summary pre-skip checkpoint では、raw initialization summary の variant-param collector を起動する前に、return value を直接 output とする top-level `Branch` の有無を判定するようにした。collector が観測できる branch がない block では prefix replay だけが走って facts が増えないため、証明能力を変えずに探索を削減する。
- 同 checkpoint の native release RPN stage-only 測定は `resource_typecheck=125ms`、`resource_initialized_i32_scalar_summaries=1232ms`、`resource_raw_init_summary_recomputations=148 summaries=78`、`resource_initialized_raw_init_summaries=2281ms`、`resource_initialized_function_checks=3090ms`、`resource_static_check=7443ms`。直前の typed public signature 測定 `resource_static_check=7776ms` から改善したが、初回 compile 0.5 秒未満にはまだ Resource IR summary cache が必要である。
- duplicate path dedup / string byte predicate checkpoint では、Resource IR initialized check の budget 超過時に完全重複 path-state だけを先に落とし、`str_trim` loop は `string/search/compare` の public `str_byte_is_ascii_space_at` predicate だけを使う形にした。sentinel 値は `string/byte_index` 内部 helper に閉じ、search predicate は byte-index 側の証明済み ASCII 空白判定を高水準 module へ渡す facade になる。
- 同 checkpoint の native release RPN stage-only 測定は `resource_initialized_i32_scalar_summaries=1256ms`、`resource_initialized_raw_init_summaries=2549ms`、`resource_initialized_function_checks=3139ms`、`resource_static_check=7870ms`。per-function timing run では `str_trim` の function check が `1018ms` から `699ms` へ下がったが、全体はまだ 7 秒台で Resource IR summary cache が必要である。
- signed integer parse checkpoint では、`to_i128_radix` が `str_slice` で signed body を作ってから `to_u128_radix` の `Result` を再度 match する形をやめた。private `parse_u128_radix_digits_from` は開始 index を受け取り、`to_u128_radix` と `to_i128_radix` が同じ digit loop と `u128_can_mul_add_small` overflow check を共有する。
- 同 checkpoint で branch / match の path alternatives と replay 対象 state を所有権移動で渡し、不要な丸ごと clone を削った。全候補への早期重複排除は equality cost で悪化したため採用せず、budget 超過時の重複排除だけを維持した。
- 同 checkpoint の native release RPN stage-only 測定は best run で `resource_initialized_i32_scalar_summaries=1172ms`、`resource_initialized_raw_init_summaries=2239ms`、`resource_initialized_function_checks=1767ms`、`resource_initialized_moves=5236ms`、`resource_static_check=6104ms`。`trunk build --release` 後の再確認では `resource_initialized_i32_scalar_summaries=1450ms`、`resource_initialized_raw_init_summaries=2647ms`、`resource_initialized_function_checks=1965ms`、`resource_initialized_moves=6139ms`、`resource_static_check=7086ms`。まだ 0.5 秒未満には届かないが、RPN の initialized function check は大きく下がった。
- i32 scalar summary local reuse checkpoint では、relevance 判定の `I32LeafProjectionCache` を共有し、複数 block 関数だけ初期 `I32ScalarPathState` を関数内で 1 回構築して block ごとに clone するようにした。単一路 return fact merge では全 path 包含確認を省き、同一 path 内の重複除去だけを行う。
- 同 checkpoint の native release RPN stage-only 測定は `resource_initialized_i32_scalar_summaries=1568ms`、`resource_initialized_raw_init_summaries=2705ms`、`resource_initialized_function_checks=1994ms`、`resource_static_check=7299ms`。前 checkpoint の `resource_static_check=7841ms` から改善したが、初回 0.5 秒未満には引き続き Resource summary value cache と raw init/function check 側の再利用が必要である。

RPN では同一入力の再compileは 10ms 未満になったが、初回 compile はまだ 0.5 秒未満から遠い。次の根本対応は raw init summary / function check の path-sensitive exploration を function hash と dependency aggregate public surface hash で再利用する Resource IR summary cache である。

2026-06-01 の NM CI timeout 調査では、`examples/nm.nepl::doctest#1` の release Web compile が
`compile_ms=16927`、`resource_static_initialized_moves=14207.1ms`、`resource_static_check=16213.979ms`
だった。native release stage timing でも `resource_initialized_function_checks` が約 7.5s、
`resource_initialized_moves` が約 12.6s を占め、`nm_inline_to_json_into`、`nm_inline_to_html`、
`document_to_json`、`str_trim` などの branch / match / loop を持つ文字列処理が上位に出た。
これは CI timeout 値だけの問題ではなく、初回 compile の Resource IR initialized-state 探索が
base compile 目標から大きく外れていることを示す。

同調査では `CellTable::availability_state_by` の ancestor / descendant clone を直接走査へ置き換える
試行も行ったが、native release の `resource_initialized_moves` が約 12.92s へ悪化したため採用しなかった。
次の候補は、単一 table の局所 clone 削減ではなく、branch / loop / match の状態 merge と
summary fixed-point を compile-local bundle / relevance / prechecked artifact 境界へ寄せることである。
CI の examples doctest は当面 `-j 2` と per-case timeout 60s で headroom を確保するが、これは
この issue の解決条件ではない。

2026-05-28 の semantic source key checkpoint で、RPN same-session の ordinary comment-only edit は `compile_ms=2`、doccomment text edit は `compile_ms=1` になった。コメント追加・修正は compiled-output cache で 10ms 未満に入ったが、code edit は `compile_ms=8347` の full compile になり、初回 / 実コード微小変更の目標は未達である。

2026-05-31 の same-session performance work で、RPN の実コード微小変更は
`tmp/rpn_i32_open_generic_reprojection_code_edit_20260531.json` において `edit compile_ms=2126`
まで下がった。一方で、同じ測定の base compile は `compile_ms=9231`、
`resource_static_check=8606.798ms` であり、0.5 秒未満の per-program compile target には
まだ届いていない。今後の性能改善では warm edit cache だけではなく、初回 compile の
stdlib prechecked artifact、Resource static check fixed-point の探索空間削減、binary
intermediate artifact 化を同時に進める必要がある。

2026-05-31 の dependency graph sharing checkpoint では、Resource static check の各 summary
kind が個別に構築していた function dependency / dependent / initial worklist order を、
compile-local な `ResourceSummaryDependencyGraph` として 1 回だけ構築して共有するようにした。
これは proof key を弱めるものではなく、同じ `ResourceModule` から導ける一時 view の重複構築を
減らす変更である。

`trunk build --release` 後の Web RPN same-session code edit 測定
`tmp/rpn_dependency_graph_share_code_edit_20260531.json` では、base `compile_ms=9246`、
`resource_static_check=8193.197ms`、unused local 追加 edit `compile_ms=2135`、
`resource_static_check=1857.811ms` だった。native release RPN stage-only 測定では
`resource_static_check=6915ms`、`resource_initialized_moves=5998ms` だった。
base compile は改善傾向だが、0.5 秒未満にはまだ大きく届かないため、この issue は
stdlib prechecked artifact / Resource proof template / binary intermediate artifact の親 issue
として open のまま維持する。

同 follow-up として、共有 `ResourceSummaryDependencyGraph` から作る `SummaryWorklist` は
`dependents` を clone せず借用するようにした。旧 constructor は owned dependents を保持するため、
既存 test helper は維持している。`tmp/rpn_borrowed_worklist_dependents_code_edit_20260531.json`
では base `compile_ms=9510`、`resource_static_check=8446.129ms`、unused local 追加 edit
`compile_ms=2251`、`resource_static_check=1943.803ms` だった。counter は dependency graph
sharing checkpoint と同じ形で、raw-init residual は増えていない。elapsed time は揺れており、
この follow-up 単体を 0.5 秒未満化の達成とは扱わない。

raw-init param facts cache staging の実測では RPN が `raw_init_param_facts_stores=0` / `bypasses=225` だった。nominal 型 identity が未整備で stdlib summary を安全に stable key 化できない問題を `ISS-20260528T110220373Z-RESOURCE-SUMMARY-CACHE-NEEDS-QUALIFI-08D1AA04` に分離した。

raw-init preseed は実際に fixed-point worklist を skip するため、function body / source policy / signature type boundary が変わる場合に stale summary を使わない regression を追加した。raw body は本文が `ResourceFunction` に残らないため、source policy hash だけではなく raw body/source hash 設計が入るまで Resource summary body hash で拒否する。

まず、代表 program を次の階層に分けて固定する。

- core only: stdlib import を含まない `#target core` の最小 return case。
- stdio only: `std/stdio` だけで 1 行出力する case。
- stream writer: `std/streamio` の writer を explicit value chain で使う case。
- stream scanner: `StreamScanner` read と stdio / writer output を組み合わせる case。
- KP workload: prefix sum, float scanner/writer, kpsearch のように stdlib graph と Resource IR summary が大きくなる case。

その上で、`NEPL_COMPILE_STAGE_TIMING=1` の native timing と wasm doctest の `timing.compile_ms` を対応付ける。候補は Resource IR summary propagation、stdlib import graph の解析範囲、diagnostic materialization、型推論候補展開、WAT/comment 補助生成である。timeout 延長、coverage 削除、旧記法のままの失敗を性能改善として扱うことはしない。

2026-05-25 の追加調査により、raw initialization summary の単純な relevance pruning は却下した。次の候補は、summary builder が reference parameter / raw address alias / initialized-cell seed をどの関数で必要とするかを明示的な fact category に分け、その category ごとに worklist を分けることである。`RawMemory` を持つ関数だけを残すのではなく、reference deref の可用性を証明する lightweight summary と raw byte/cell mutation summary を分ける必要がある。

2026-06-02 の content-addressed stdlib dependency aggregate cache checkpoint では、Web bundled stdlib の
session cache namespace を stdlib 全体の content hash として扱い、同一 session 内の dependency
aggregate public surface query を path/source hash で再利用できるようにした。これは stdlib overlay を
含む mutable provider には適用せず、closed-source stdlib でだけ child closure traversal を省く境界である。

`tmp/artifact-closed-source-aggregate-cache-20260602-r3.json` では、`core/char` materialized compile fallback
bench の cold base が `compile_ms=423`、warm store probe が `compile_ms=230`、body edit candidate が
`compile_ms=206`、body edit repeat が `compile_ms=185` になった。直前の fragment insertion checkpoint の
cold base `compile_ms=1477` / body edit repeat `compile_ms=758` からは改善したが、目標である
base 0.5 秒未満と edit 0.1 秒未満を一般 case で達成したとは扱わない。次は bundled stdlib
`.neplmeta` / `.neplproof` preseed、`.neplobj` same-session object store、Resource proof template を
継続する。

## 検証

- `trunk build`
- `node nodesrc/run_doctest.js -i tmp/agent_perf_cases_20260525.n.md -n 1 --dist web/dist`
- `node nodesrc/run_doctest.js -i tmp/agent_perf_cases_20260525.n.md -n 2 --dist web/dist`
- `node nodesrc/run_doctest.js -i tmp/agent_perf_cases_20260525.n.md -n 3 --dist web/dist`
- `node nodesrc/run_doctest.js -i tmp/agent_perf_cases_20260525.n.md -n 4 --dist web/dist`
- `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 1 --dist web/dist`
- `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 3 --dist web/dist`
- `node nodesrc/tests.js -i tests/stdlib/kp.n.md --no-tree -o tmp/agent_perf_kp_20260525.json -j 1 --assert-io --dist web/dist`
