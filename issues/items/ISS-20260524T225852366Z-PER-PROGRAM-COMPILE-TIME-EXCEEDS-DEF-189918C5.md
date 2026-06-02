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

2026-06-02 の `.neplmeta` / `.neplobj` fallback root-cause checkpoint では、同じ `core/char`
fixture の warm edit が 0.1 秒未満に入った。`tmp/materialized-raw-wasm-neplobj-20260602-rerun.json`
では cold base `compile_ms=387`、warm store probe `compile_ms=221`、body edit candidate
`compile_ms=21`、body edit repeat `compile_ms=21` である。

この改善は、nominal placeholder hash、loader artifact staging、source path based source definition
skip、raw wasm leaf fragment、relocation dependency closure を合わせて、materialized compile の
non-body-missing fallback を消した結果である。`materialized_non_body_missing_fallbacks_delta_sum=0`
になり、fallback code は `backend.codegen.materialized_function_body_missing` だけになった。

ただし、この issue は解決しない。対象 fixture の base は 0.5 秒未満に入ったが、RPN / stdlib-heavy
workload の base compile と 10ms 級 expression subtree edit はまだ未達である。次の作業は、bundled
stdlib `.neplmeta` / `.neplproof` preseed、persistent `.nepl...` codec、changed-expression query、
Resource proof template、generic / string-data / raw body `.neplobj` fragment の順に進める。

2026-06-02 の RPN cold base 再測定では、`examples/rpn.nepl` を代表 workload として固定した。

- native release `target\release\nepl-cli.exe --check -i examples\rpn.nepl --target std --stdlib-root stdlib`
  は成功し、`resource_typecheck=147ms`、`resource_static_check=6172ms`、
  `resource_initialized_moves=5320ms`、`resource_initialized_i32_scalar_summaries=1330ms`、
  `resource_initialized_raw_init_summaries=2315ms`、`resource_initialized_function_checks=1609ms`、
  `resource_owner_obligations=736ms` だった。
- release Web cold base は `NEPL_RUN_TEST_SKIP_COMPILER_WARMUP=1` の `nodesrc/run_test.js` で
  `compile_ms=9283`、`wasm_call_ms=8640`、`total_ms=9313` だった。stage timing では
  `resource_static_initialized_moves=6626.938ms`、`resource_static_owner_obligations=1455.416ms`、
  `resource_static_check=8237.881ms` が支配的である。
- per-function timing では `str_trim` final initialized check が約 0.8-0.9 秒、
  `sb_append_result` i32 scalar summary が約 0.65-0.85 秒、`apply_op` / `dealloc_raw` /
  `byte_builder_*` raw-init summary が 0.1-0.5 秒台に分散している。
- `str_trim` の op timing では loop / branch の clone / merge が支配的であり、
  path-sensitive alternatives の指数増殖は確認されなかった。
- enum variant projection の不可能 leaf を i32 scalar return fact 収集時に削る試行は、
  native RPN の `resource_static_check` が `7455ms` へ悪化したため採用しない。
- branch / loop merge 前の `CellTable` / raw alias / slot table clone を借用版 merge API へ置き換える
  試行も、native RPN の `resource_static_check=7933ms`、`resource_initialized_moves=6949ms` へ
  悪化したため採用しない。局所 clone 削減ではなく、Resource proof の初回構築固定費を
  artifact preseed で避ける必要がある。

この結果から、RPN cold base を 0.5 秒未満へ近づける主経路は、局所 leaf pruning ではなく、
stdlib-heavy Resource proof を初回 compile 前から利用できる bundled / persistent `.neplproof`
preseed である。既存の in-memory `ResourceSummaryProofArtifact` は header 照合と same-session
preseed を持つが、`ResourceSummaryProofSnapshot` は serialization schema ではない。次の実装は
`.neplproof` stable entry codec、header first decode、generic type-argument key、source capability
policy set hash、private effect policy hash を fail-closed に固定し、disk / IndexedDB / build-time
bundled artifact へ接続する。

2026-06-02 の追加 profiling では、`sb_append_result` の i32 scalar summary が return fact 収集で
重く、`I32ConditionQueryContext` の memo が `Vec` 線形探索になっていることを確認した。`Place` は
`Ord` を持つため、同じ純粋 query の memo を `BTreeMap` 化し、検査規則を変えず探索構造だけを
改善した。native RPN では `resource_initialized_i32_scalar_summaries=1421ms / 1189ms`、
`resource_static_check=7389ms / 7053ms` まで下がり、`sb_append_result` の `collect_facts` は
`684ms / 159ms` から `374ms / 109ms` へ改善した。ただし raw-init、`str_trim` final check、
owner summary の固定費が残るため、この issue は open のまま維持する。

2026-06-02 の追加 followup では、`str_trim` と `apply_op` の op timing から、path-sensitive replay
増殖ではなく `CellTable::availability_state_by` の control-flow merge 固定費を削った。ancestor /
descendant entry の一時 `Vec` clone と複数回走査を、同じ優先順位を保つ allocation-free な単一走査へ
置き換えた。native release RPN stage-only は同一 followup baseline の `resource_static_check=7865ms`
から、変更後の単独計測で `6267ms / 6381ms` まで下がった。`resource_initialized_moves` は
`6756ms` から `5208ms / 5484ms`、per-function timing では `str_trim` final check が `895ms`、
`apply_op` raw-init summary が `552ms` になった。

remote/main の GUI 関連変更を取り込んだ main 上で release CLI を再ビルドした後の確認値は、
`resource_static_check=6937ms`、`resource_initialized_moves=5876ms`、
`resource_initialized_raw_init_summaries=2378ms`、`resource_initialized_function_checks=1905ms` だった。

これは探索構造の改善であり、base 0.5 秒未満を達成するものではない。RPN / stdlib-heavy workload の
主経路は引き続き bundled / persistent `.neplproof` preseed、stdlib proof template、changed-expression
query cache である。

2026-06-02 の `str_trim` scan helper split checkpoint では、remote/main 同期後の native release
RPN baseline を再測定した。同期後の stage-only run は `resource_static_check=7565ms` / `7455ms`、
`resource_initialized_moves=6452ms` / `6351ms`、`resource_initialized_function_checks=2014ms` /
`1974ms` で、per-function timing では `str_trim__str__str__pure` の final initialized check が
`1117ms` だった。

対応として、`str_trim` の先頭側 scan と末尾側 scan を `str_trim_left_index` /
`str_trim_right_index` の private helper へ分け、public `str_trim` は `len`、左右 index、
`str_slice` を接続するだけにした。公開 API、空白判定、範囲確認境界、allocation failure の扱いは
変えていない。変更後の native release RPN stage-only 後続 run は `resource_static_check=5787ms` /
`5397ms`、`resource_initialized_moves=4742ms` / `4514ms`、
`resource_initialized_function_checks=1029ms` / `921ms` だった。per-function timing では `str_trim`
が上位 50 件から外れた。

この改善でも RPN cold base は 0.5 秒未満から遠いため、この issue は open のまま維持する。残る
支配点は `apply_op` / `dealloc_raw` の raw-init summary、`sb_append_result` の i32 scalar summary、
および stdlib-heavy Resource proof を初回 compile ごとに構築する固定費である。次の根本対応は
bundled / persistent `.neplproof` preseed と stdlib proof template である。

2026-06-02 の RPN operator / builder helper split checkpoint では、`str_trim` 分割後の
`examples/rpn.nepl` cold base をさらに分解した。変更前の同 branch baseline は
`resource_static_check=5870ms` / `5900ms`、`resource_initialized_moves=4897ms` / `5011ms`、
`resource_initialized_i32_scalar_summaries=1258ms` / `1273ms`、
`resource_initialized_raw_init_summaries=2560ms` / `2655ms` だった。per-function timing では
`apply_op__Stack_T_i32_str...` raw-init summary が `611ms`、`dealloc_raw` raw-init summary が
`520ms`、`sb_append_result` i32 scalar summary が `441ms` だった。

対応として、RPN operator token を先に `RpnOp` enum へ分類する `operator_from_token` を追加し、
`apply_op` は分類済み `RpnOp` を受け取るようにした。演算選択は pure helper `apply_op_values` へ分け、
Stack owner の pop / push と文字列比較 chain を同じ関数へ集中させない。StringBuilder 側は
`sb_append_non_empty_result` と `sb_byte_builder_error_text` を追加し、public `sb_append_result` を
空文字 fast path と非空 append path の接続に絞った。

変更後の native release RPN stage-only run は `resource_static_check=5372ms` / `4927ms`、
`resource_initialized_moves=4340ms` / `4013ms`、`resource_initialized_i32_scalar_summaries=1135ms` /
`1090ms`、`resource_initialized_raw_init_summaries=2309ms` / `2047ms`、
`resource_initialized_function_checks=824ms` / `801ms` だった。per-function timing run は
`resource_static_check=5453ms` で、hot path は `dealloc_raw` raw-init `498ms`、`apply_op` raw-init
`426ms`、`sb_append_non_empty_result` i32 scalar `320ms`、`byte_builder_push_bytes_ref` raw-init `232ms`、
`byte_builder_reserve` i32 scalar `183ms` の順である。

`apply_op` raw-init は `611ms` から `426ms` へ下がり、`sb_append_result` wrapper は `3ms` まで
小さくなった。ただし残る支配点は `dealloc_raw` / ByteBuilder / Stack owner flow の proof と
stdlib-heavy Resource proof の初回構築であるため、この issue は open のまま維持する。base compile
0.5 秒未満には、引き続き actual `.neplproof` artifact の bundled / persistent preseed と stdlib
proof template が必要である。

2026-06-02 の RPN summary fixed-point index checkpoint では、前 checkpoint 後も残っていた
Resource summary 固定点中の `function -> summary position` 索引再構築を削った。`SummaryNameIndex`
を追加し、raw alias / i32 scalar / raw-init / collection-slot / raw pointer / raw identity / owner
summary の反復中に索引を保持する。summary の追加・削除時だけ索引を更新し、各 summary kind の
`SummaryIndex` はこの索引を借用 view として使う。あわせて raw memory release requirement の対象
引数表を毎回 `Vec` にせず、静的 slice として返すようにした。

release CLI 再ビルド後の native stage-only run は `resource_static_check=6314ms / 5972ms / 5899ms`、
`resource_initialized_moves=5169ms / 4914ms / 4776ms`、
`resource_initialized_raw_init_summaries=2866ms / 2592ms / 2572ms`、
`resource_initialized_i32_scalar_summaries=1198ms / 1227ms / 1121ms`、
`resource_initialized_function_checks=1028ms / 1010ms / 996ms`、
`resource_owner_obligations=986ms / 903ms / 968ms` だった。

同じ release rebuild 前の clean baseline は `resource_static_check=6622ms / 6689ms`、
`resource_initialized_moves=5416ms / 5455ms`、`resource_initialized_raw_init_summaries=2814ms`、
`resource_initialized_i32_scalar_summaries=1406ms / 1428ms`、
`resource_owner_obligations=1032ms / 1061ms` だった。このため、今回の索引化は検査規則を変えずに
RPN cold static-check cost を約 5-12% 削ったが、0.5 秒未満目標にはまだ大きく届かない。

関数別 profiling run はログ出力 overhead を含むため総量比較には使わないが、残る hot spot は
`dealloc_raw` raw-init summary、`apply_op` raw-init summary、`byte_builder_push_bytes_ref` /
`byte_builder_reserve` / `byte_builder_push_u8` raw-init summary、`sb_append_non_empty_result`
i32 scalar summary だった。filtered sequential timing では `dealloc_raw` raw-init summary の
`release_requirements` が `570ms` を占め、filtered run の `apply_op` は `release_requirements=380ms`、
`variant_param_cells=117ms` だった。RPN cold base をさらに大きく下げるには、関数名索引のような
局所構造改善だけではなく、raw-init release requirement の control-flow replay と stdlib-heavy
Resource proof を `.neplproof` persistent / bundled preseed へ移す必要がある。

2026-06-02 の RPN release requirement state-step checkpoint では、raw-init release requirement
collector が `Branch` / `Match` body を obligation 収集と full `check_ops` replay の両方で重複して
歩いていたことを確認した。初期計測では `dealloc_raw` の op timing が `branch count=15 total=1225ms`
と `call count=144 total=647ms` に偏っていた。

対応として、release requirement collector に `Branch` / `Match` 専用の state step を追加した。
nested body の release obligation は再帰で収集し、後続 sibling op に必要な `CellTable`、
`CollectionSlotStateTable`、raw alias、function alias、pending realloc、variant initialization だけを
実際の Resource check と同じ転送規則で進める。これにより、検査規則を弱めずに raw-init summary の
control-flow replay を削減した。`CollectionSlotStateTable` は再帰にも渡し、control value transfer が
collection slot lifecycle proof を失わないようにした。

あわせて i32 scalar return fact 収集では、raw alias 側に i32 value condition を証明できる source が
ない場合に parameter / return condition の全候補探索を省く guard を追加した。直接定数と offset
constant endpoint は保持するため、既存の condition proof 能力は維持している。

最終形の native release RPN stage-only 3 run は `resource_static_check=3941ms / 3595ms / 3878ms`、
`resource_initialized_moves=2958ms / 2648ms / 2917ms`、
`resource_initialized_raw_init_summaries=918ms / 821ms / 935ms`、
`resource_initialized_i32_scalar_summaries=1116ms / 997ms / 1076ms`、
`resource_initialized_function_checks=849ms / 765ms / 833ms`、
`resource_owner_obligations=847ms / 824ms / 831ms` だった。baseline median `resource_static_check=5406ms`
から final median `3878ms` へ約 28% 下がり、raw-init summary median は `2325ms` から `918ms` へ下がった。

最終形の hotspot は、i32 scalar summary では `sb_append_non_empty_result=269ms`、
`byte_builder_reserve=156ms`、`byte_builder_push_bytes_ref=96ms`、raw-init summary では
`apply_op=189ms`、`dealloc_raw=164ms`、function check では `dealloc_raw=162ms`、
`parse_u128_radix_digits_from=97ms`、`apply_op=93ms` である。`dealloc_raw` の最終 op timing は
`branch count=6 total=288ms`、`call count=86 total=194ms` まで下がった。

この checkpoint でも issue は解決しない。RPN cold base はまだ 0.5 秒未満ではなく、次の根本対応は
ByteBuilder / StringBuilder 系 i32 scalar condition proof の探索空間削減、`dealloc_raw` / `apply_op`
/ Stack owner flow の Resource proof template 化、bundled / persistent `.neplproof` preseed である。

2026-06-02 の RPN i32 scalar variant projection filter checkpoint では、`Result` の Ok / Err のような
concrete variant で到達不能な sibling payload leaf まで i32 scalar return fact 収集が開始されることを
確認した。direct parameter condition の path 間 intersection は
`resource_initialized_i32_scalar_summaries=1314ms / 1270ms / 1062ms` へ悪化したため採用していない。

採用した対応は、return leaf 収集前に `state.concrete_variants.projection_is_possible` で不可能な projection
を除外する filter である。variant が不明な場合は fail-open で従来通り全 leaf を探索するため、
静的検査の意味は弱めない。focused test では、到達不能 leaf の constant fact 削減、fail-open、
parameter condition 保持、到達可能 leaf 同士の offset 由来 relation 保持を固定した。

変更後の native release RPN stage-only 3 run は `resource_static_check=3834ms / 3793ms / 3684ms`、
`resource_initialized_moves=2897ms / 2850ms / 2763ms`、
`resource_initialized_i32_scalar_summaries=1107ms / 1040ms / 993ms`、
`resource_initialized_raw_init_summaries=894ms / 901ms / 893ms`、
`resource_initialized_function_checks=817ms / 839ms / 804ms`、
`resource_owner_obligations=804ms / 805ms / 792ms` だった。直前 checkpoint median から
`resource_static_check=3878ms -> 3793ms`、`resource_initialized_i32_scalar_summaries=1076ms -> 1040ms`
への小幅改善であり、issue の 0.5 秒未満目標にはまだ届いていない。

per-function timing では i32 scalar summary の上位が `sb_append_non_empty_result=317ms`、
`byte_builder_reserve=189ms`、`byte_builder_push_bytes_ref=118ms`、`apply_op=112ms`、
`byte_builder_push_u8=100ms` だった。raw-init summary は `apply_op=195ms`、`dealloc_raw=164ms`、
function check は `dealloc_raw=175ms`、`parse_u128_radix_digits_from=106ms`、`apply_op=100ms` が
残っている。次の根本対応は actual `.neplproof` preseed を native `--check` cold path に接続することと、
stdlib-heavy proof の事前検査済み artifact 化である。

2026-06-02 の RPN source-specific proof gate checkpoint では、native `nepl-cli --check` が Web
`CompilerSession` と違って `ResourceSummaryValueCache` / `.neplproof` preseed を使っていないことを
確認した。空の `ResourceSummaryValueCache` を native check path に接続する試行は
`resource_static_check=6761ms / 6849ms / 6816ms` へ悪化したため採用していない。preseed hit が無い状態で
cache key / replay probe だけを追加しても cold base は速くならない。

採用した対応は、raw-init release requirement の parameter alias list を summary application ごとに一度だけ
作ることと、i32 scalar condition 探索の前提判定を対象 value の scalar aliases に届く fact へ狭めることである。
無関係な i32 fact が同じ raw alias graph にあるだけでは cold leaf の condition 探索を開始しない。

formatter 後の native release RPN stage-only 3 run は `resource_static_check=3979ms / 3433ms / 3538ms`、
`resource_initialized_moves=2973ms / 2559ms / 2578ms`、
`resource_initialized_i32_scalar_summaries=1035ms / 891ms / 860ms`、
`resource_initialized_raw_init_summaries=976ms / 831ms / 896ms`、
`resource_initialized_function_checks=883ms / 770ms / 757ms`、
`resource_owner_obligations=865ms / 754ms / 839ms` だった。median `resource_static_check=3538ms` まで
下がったが、0.5 秒未満目標にはまだ届かない。

`origin/main` `9812d619` を取り込んで main へ merge した後の再測定では、
`resource_static_check=4083ms / 4177ms / 4082ms`、
`resource_initialized_moves=3039ms / 3138ms / 3095ms`、
`resource_initialized_i32_scalar_summaries=1065ms / 1087ms / 1034ms`、
`resource_initialized_raw_init_summaries=998ms / 1030ms / 1037ms`、
`resource_initialized_function_checks=896ms / 934ms / 942ms`、
`resource_owner_obligations=894ms / 896ms / 848ms` だった。post-merge median の
`resource_static_check=4083ms` も RPN cold base の現行基準として扱う。

per-function timing の階層は、`resource_static_check=3723ms` のうち
`resource_initialized_moves=2705ms`、その内訳として i32 scalar `929ms`、raw-init `890ms`、function check
`811ms`、さらに owner obligations `881ms` が残る形である。上位関数は i32 scalar が
`sb_append_non_empty_result=268ms`、`byte_builder_reserve=173ms`、raw-init が `apply_op=185ms`、
`dealloc_raw=148ms`、function check が `dealloc_raw=151ms`、`parse_u128_radix_digits_from=91ms`、
`apply_op=80ms` である。次の根本対応は bundled / persistent `.neplproof` preseed、stdlib proof template、
owner return summary stable mirror である。

2026-06-02 の RPN cold base follow-up では、HEAD 相当の native release `--check` を再ビルドし、
stage-only 3 run と per-function timing を取り直した。stage-only は
`resource_static_check=6303ms / 4993ms / 4115ms`、`resource_initialized_moves=4754ms / 3383ms / 3154ms`、
`resource_initialized_i32_scalar_summaries=1114ms / 1207ms / 1118ms`、
`resource_initialized_raw_init_summaries=2390ms / 1086ms / 1052ms`、
`resource_initialized_function_checks=1164ms / 1005ms / 895ms`、
`resource_owner_obligations=1400ms / 1445ms / 816ms` だった。

per-function timing 1 run の階層は `resource_static_check=4358ms`、
`resource_initialized_moves=3293ms`、`resource_owner_obligations=923ms` である。initialized moves の内訳は
i32 scalar `1177ms`、raw-init `1053ms`、function check `959ms`。関数別上位は i32 scalar が
`sb_append_non_empty_result=344ms`、`byte_builder_reserve=239ms`、`apply_op=103ms`、raw-init が
`apply_op=213ms`、`dealloc_raw=186ms`、function check が `dealloc_raw=168ms`、
`parse_u128_radix_digits_from=111ms`、`apply_op=101ms` だった。

この checkpoint で試した local optimization は採用していない。condition-candidate memo は
`Place` key の `BTreeMap` 固定費が大きく、offset 由来 parameter condition の重複削減は focused test
通過後も RPN per-function で悪化し、owner summary relevance filter は recomputation を `295 -> 267` に
減らしても総時間を改善しなかった。したがって、この issue の次作業は local memo ではなく
actual `.neplproof` stable codec、native `--check` preseed、stdlib proof template、owner return summary
stable mirror へ絞る。

2026-06-02 の proof-backed check gate checkpoint では、native cold `--check` に空の
`ResourceSummaryValueCache` を接続して悪化させる経路を防ぐため、`ResourceSummaryValueCacheActivation`
を追加した。既定の `Always` は Web `CompilerSession` などの same-session cache 収集を維持する。
`OnlyAfterAcceptedPreseed` は `.neplproof` preseed report が usable entry を持つ場合だけ Resource static
check へ cache を渡し、artifact missing / compatibility reject / empty artifact では cache と context を
渡さない。

この checkpoint の RPN release stage-only 3 run は、変更前確認が
`resource_static_check=3785ms / 3359ms / 3563ms`、変更後が
`resource_static_check=3882ms / 3856ms / 3563ms` だった。通常 CLI path は baseline 範囲内だが、
0.5 秒未満目標にはまだ届いていない。この変更は actual `.neplproof` を速くするものではなく、次の
persistent / bundled proof artifact を native `--check` cold path へ安全に接続するための境界である。

2026-06-02 の RPN allocator / ByteBuilder reserve helper split checkpoint では、proof-backed check gate 後の
native release RPN を再測定した。作業開始時点の stage-only 3 run は
`resource_static_check=3888ms / 3600ms / 4158ms`、中央値 `3888ms` である。per-function timing では
`dealloc_raw` が raw-init summary と final initialized function check の両方に残り、
`byte_builder_reserve` が i32 scalar summary 上位に残っていた。

対応として、`stdlib/core/mem/allocator.nepl` の `dealloc_raw` を free list 挿入位置探索、link、
next coalesce、prev coalesce の private helper へ分けた。address-order free list、`ptr <= 0` no-op、
前後 coalesce、runtime ABI は変えていない。`stdlib/alloc/io/bytebuilder/storage.nepl` では
`byte_builder_reserve` の Empty storage grow と Owned storage grow を private helper へ分けた。失敗時の
builder owner recovery、capacity exceeded / out-of-memory / invalid operation の分類、旧 region token
recovery は維持している。

変更後の native release RPN stage-only 5 run は
`resource_static_check=3134ms / 2819ms / 3054ms / 2801ms / 3018ms`、中央値 `3018ms` である。中央値内訳は
`resource_initialized_moves=2248ms`、`resource_initialized_i32_scalar_summaries=783ms`、
`resource_initialized_raw_init_summaries=741ms`、`resource_initialized_function_checks=651ms`、
`resource_owner_obligations=607ms` だった。native CLI 全体 elapsed は `Measure-Command` で約 `3720ms`。
per-function timing では `byte_builder_reserve` i32 scalar summary が約 `166ms` から約 `36ms` へ下がり、
`dealloc_raw` は上位 hot path から大きく後退した。

この checkpoint でも issue は解決しない。RPN cold base は秒単位であり、0.5 秒未満へ進める主経路は
actual `.neplproof` preseed、stdlib proof template、owner return summary stable mirror のままである。

2026-06-02 の RPN Stack pop2 owner-flow checkpoint では、proof-backed check gate と allocator /
ByteBuilder reserve helper split 後の RPN cold base を再測定した。作業開始時点の native release
stage-only 3 run は `resource_static_check=2995ms / 2939ms / 2813ms`、中央値 `2939ms` である。
支配階層は次の通り。

```text
RPN cold base static check before Stack pop2
  resource_static_check: median 2939ms
    resource_initialized_moves: 2271ms / 2173ms / 2116ms
      resource_initialized_i32_scalar_summaries: 836ms / 744ms / 737ms
      resource_initialized_raw_init_summaries: 734ms / 700ms / 680ms
      resource_initialized_function_checks: 636ms / 666ms / 637ms
    resource_owner_obligations: 602ms / 640ms / 581ms
```

対応として、`StackPop2` と `pop_top2` を `stdlib/alloc/collections/stack` に追加し、RPN の
`apply_op` が 2 回の `pop_top` で Stack owner を順に移動する形を 1 つの owner boundary へ集約した。
2 要素未満では stack owner を変更せず `None` を返すため、underflow path でも呼び出し側の owner
recovery が明示的である。

変更後の native release stage-only 5 run は次の通り。

```text
RPN cold base static check after Stack pop2
  resource_static_check: 3223ms / 2877ms / 2882ms / 2611ms / 2833ms
    median: 2877ms
    resource_initialized_moves median-near: about 2092ms
      resource_initialized_i32_scalar_summaries median-near: about 783ms
      resource_initialized_raw_init_summaries median-near: about 611ms
      resource_initialized_function_checks median-near: about 631ms
    resource_owner_obligations median-near: about 650ms
```

同時に試した ByteBuilder reserved write helper は、`byte_builder_push_bytes_ref` /
`byte_builder_push_u8` の owner recovery が helper boundary をまたいで
`resource.owner.use_after_move` になるため採用しなかった。検査を弱めて通すのではなく、ByteBuilder の
残支配点は actual `.neplproof` preseed、stdlib proof template、owner return summary stable mirror で扱う。

## 検証

- `trunk build`
- `node nodesrc/run_doctest.js -i tmp/agent_perf_cases_20260525.n.md -n 1 --dist web/dist`
- `node nodesrc/run_doctest.js -i tmp/agent_perf_cases_20260525.n.md -n 2 --dist web/dist`
- `node nodesrc/run_doctest.js -i tmp/agent_perf_cases_20260525.n.md -n 3 --dist web/dist`
- `node nodesrc/run_doctest.js -i tmp/agent_perf_cases_20260525.n.md -n 4 --dist web/dist`
- `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 1 --dist web/dist`
- `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 3 --dist web/dist`
- `node nodesrc/tests.js -i tests/stdlib/kp.n.md --no-tree -o tmp/agent_perf_kp_20260525.json -j 1 --assert-io --dist web/dist`
