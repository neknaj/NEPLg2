# NEPLg2.1 compiler performance / cache design 2026-05-27

## 位置づけ

この文書は、NEPLg2.1 移行中の compile-time performance 改善方針を固定する。対象は Rust 実装の `nepl-core`、Web/WASM wrapper の `nepl-web`、Node test runner、stdlib prelude/import graph である。

2026-05-27 に Zenn の「試作段階における開発方針」を再確認した。現段階では後方互換よりも設計の正しさを優先する。ただし静的検査の正確性、source capability、Resource IR の安全性証明を削って速度を得ることはしない。timeout 延長、検査削除、coverage 削減は performance fix として扱わない。

## 目標

- 通常の 1 program compile は 0.5 秒未満にする。
- release WASM artifact を使う warm session では、最小規模の微小再compileを 10ms 未満にする。
- stdlib は、generic の具体化以外をできるだけ事前検査済み artifact として扱い、通常 compile では link / instantiate に近い形に寄せる。
- cache は純粋な query result として扱い、入力 hash、compiler version、target、profile、stdlib artifact hash を key にする。

## CI timeout headroom

CI の timeout や worker 並列度は、性能改善そのものではない。現在の `examples/nm.nepl` は
初回 compile の `resource_static_initialized_moves` が 10 秒台を占めるため、GitHub Actions の
`examples-test` では wasm compiler worker を過剰に並列実行すると 20 秒の per-case timeout に
近づきすぎる。

2026-06-01 の checkpoint では、`examples-test` を `-j 2`、`NEPL_TEST_CASE_TIMEOUT_MS=60000`
へ変更し、現在の base compile が不安定な CI failure で隠れないようにした。この変更は
0.5 秒未満 compile 目標の達成ではなく、`ISS-20260524T225852366Z-PER-PROGRAM-COMPILE-TIME-EXCEEDS-DEF-189918C5`
の継続作業中に CI の観測面を保つための headroom である。

## 判明した根本原因

### Resource IR が未到達 stdlib 関数まで検査していた

NEPLg2.1 の static-check 強化後、monomorphize が runtime helper や stdlib function を広く残し、Resource IR summary builder が entry から到達しない関数まで固定点計算していた。これにより、最小 program でも raw init summary / initialized check が数十秒規模になっていた。

対策として、Resource IR static check の直前で HIR call graph を entry から辿り、到達関数だけを残す。`CallIndirect`、曖昧な mangled prefix、raw LLVM body、raw wasm direct call のように call graph が閉じない場合は conservative-all に倒し、検査漏れを避ける。

### default prelude が allocator graph を暗黙 import していた

`std/prelude_base` は `core/traits/copy` を通じて `core/mem` 全体を読み、`core/mem` は allocator / raw memory / pointer wrapper まで引いていた。最小 program でも heavy stdlib graph が import され、typecheck と Resource IR の入力が大きくなっていた。

対策として、primitive `Copy` / `Clone` 定義を `core/traits/copy/primitive` に分離し、`core/traits/copy` facade は `MemPtr .T` impl のために `core/mem/types` だけへ依存する。さらに `core/mem/types` は compiler memory type definition と軽い metadata だけを持ち、bounds check は `core/mem/pointer/region` へ移した。これにより default prelude は `MemPtr` capability 互換を保ちながら allocator graph を読まない。

### Node/WASM test runner が毎回 full stdlib VFS を渡していた

`nodesrc/run_test.js` は local stdlib を毎回 JS object として WASM API へ渡していた。WASM 側はその object を走査し、`BTreeMap<PathBuf, String>` へ詰め直すため、compiler 本体の改善後も test runner 境界に overhead が残った。

対策として、`trunk build` artifact の mtime と local `stdlib/**/*.nepl` の newest mtime を比較し、artifact が新しければ bundled stdlib を使う。local stdlib が artifact より新しい場合だけ FS stdlib VFS を渡す。`run_test` の JSON timing には `stdlib_vfs_mode`、`stdlib_vfs_ms`、`wasm_call_ms`、`warmup_ms` を出し、どの層が支配的かを追えるようにした。

## 現在の測定値

測定日は 2026-05-27。対象は現在 branch の実装である。

| case | command / artifact | result |
|---|---|---|
| native minimal check | `target/debug/nepl-cli.exe --check -i tmp/minimal_perf.nepl --target wasm --stdlib-root stdlib` | elapsed 160ms、`resource_typecheck=5ms`、`resource_static_check=1ms` |
| native aggregate check | `target/debug/nepl-cli.exe --check -i tmp/perf_alloc_probe.nepl --target wasm --stdlib-root stdlib` | elapsed 166ms、`resource_typecheck=6ms`、`resource_static_check=1ms` |
| Web release cold minimal | `NEPL_RUN_TEST_SKIP_COMPILER_WARMUP=1` + `nodesrc/run_test.js` | `compile_ms=231`、`total_ms=257`、`stdlib_vfs_mode=bundled` |
| Web release warm minimal | `nodesrc/run_test.js` default warmup | `compile_ms=5`、`total_ms=254`、`warmup_ms=222` |
| Web release warm aggregate | `nodesrc/run_test.js` default warmup | `compile_ms=22`、`total_ms=247`、`warmup_ms=191` |

release WASM では、最小 program の warm compile が 10ms 未満になった。一方で aggregate/generic/stdlib-heavy case の微小変更を常に 10ms 未満にするには、source-level warmup だけでは不足する。次段階では CompilerSession と stdlib prechecked artifact が必要である。

## 実装済みの境界

- Resource IR 前の entry reachability pruning。
- unknown call graph では pruning を行わない conservative-all fallback。
- `Copy` / `Clone` primitive module と `core/traits/copy` facade の責務分離。
- `core/mem/types` の軽量化と `region_in_bounds` の pointer region module への移動。
- WASM wrapper の bundled stdlib lookup を `&'static str` source table + overlay source に分離。
- Node runner の stdlib VFS freshness 判定、warm compile、timing metadata。
- Resource IR の per-function timing instrumentation は環境変数 `NEPL_RESOURCE_PER_FUNCTION_TIMING` で明示した時だけ出す。
- `nepl-web` に `CompilerSession` wasm-bindgen class を公開し、WASM instance 内で bundled stdlib source table を保持する。
- Node runner は `CompilerSession` が利用できる場合に session API を優先し、JSON timing に `compiler_session` を出す。
- `nepl-web` build artifact に bundled stdlib content hash を埋め込み、Node runner の stdlib freshness 判定は hash を優先する。
- Web playground worker と tutorial runtime は method 単位で `CompilerSession` を優先し、旧 artifact で該当 session API がない場合だけ stateless / full stdlib VFS fallback を使う。
- Web playground の compile request は editor 用 read-only stdlib files や runtime data files を overlay VFS へ含めず、user-editable `.nepl` source だけを worker へ渡す。WASI 実行用の VFS snapshot は compile overlay と分離して保持する。
- `nepl-core::loader` に `LoaderSessionCache` を追加し、provider-backed load では bundled stdlib の raw parsed module を session 内で再利用する。
- parsed stdlib module cache は `cache version + stdlib namespace hash + canonical path + source hash + imported type arity hint hash` を key にし、`SourceMap` / typed HIR / `TypeId` を保存しない。
- cached AST は中立 `FileId` に正規化して保持し、compile ごとの fresh `SourceMap` で採番された `FileId` へ再投影してから merged module へ使う。
- stdlib override / overlay が `/stdlib` 以下を差し替える場合は parsed module cache を bypass し、bundled stdlib 用 artifact を local override へ混ぜない。
- `LoaderSessionCache` は source arity surface cache も保持する。value は local type arity hints、prelude/import/include/public re-export の resolved path、root-only default prelude 判定だけであり、`FileId` / `Span` / `ImportResolution` / typed HIR / `TypeId` は保存しない。
- source arity surface は `cache version + stdlib namespace hash + stdlib root + canonical path + source hash` を key にする。public re-export 先の arity result は親 surface へ畳み込まず、依存先 source hash の別 query として再評価する。
- Web playground の `trunk build` は Rust/WASM artifact を release profile で生成する。NEPL source 側の `#if[profile=...]` 既定値は `debug` のまま分離し、compiler artifact の最適化状態で source semantics が変わらないようにする。
- `CompilerSession` は同一 source / VFS / profile / WAT comment mode の compiled output を小さな LRU 風 cache に保持する。これは同一 session の再compileでResource IR全体を再実行しないための応急的な出力cacheであり、typed public surface / Resource IR summary の semantic incremental cache を置き換えるものではない。
- `PreparedProgram` は `ResourceSummaryCacheNamespaceKey` を保持する。これは target / profile / typed public signature hash / dependency public surface hash option から決定的に作る module-level namespace key であり、Resource IR summary value の再利用はまだ行わない。
- `stdlib/alloc/string/integer/parse.nepl` の signed integer parse は、`str_slice` で一時 signed body を構築してから `to_u128_radix` の `Result` を再度 match する形をやめ、private digit parser を開始 index 指定で共有する。これは stdlib 実装を軽くするだけでなく、Resource IR の branch / match / owner-state exploration を減らす。
- Resource path-state replay では、branch / match 後に保持する alternatives と replay 対象 alternatives を所有権移動で渡し、丸ごと clone を避ける。重複排除を常時行う案は equality cost が高く、budget 超過時のみに留める。

## RPN signed integer parse checkpoint

2026-05-28 の checkpoint では、Resource summary value cache の本体実装へ入る前に、RPN workload で実測上重かった stdlib integer parse と path-sensitive replay の clone を整理した。

`to_i128_radix` の旧実装は、先頭 `-` を見つけると `str_slice s 1 n` で本文を作り、`to_u128_radix` を呼んでから `Result` を match していた。この形は実行時には単純でも、Resource IR 静的検査では slice construction、callee summary、Result branch、i128 範囲判定が重なり、`to_i128_radix` 自体の initialized function check が RPN の上位コストになっていた。

新しい境界:

- `parse_u128_radix_digits_from s radix start` は private helper である。
- `to_u128_radix` は `validate_radix` 後に `start=0` で helper を呼ぶ。
- `to_i128_radix` は `validate_radix` 後、負号がある場合だけ `start=1` にして helper を呼ぶ。
- helper は `u128_can_mul_add_small` による overflow check を `u128_mul_small` / `u128_add_small` の前に行う。
- `to_i128_radix` は helper の magnitude に `i128_magnitude_allowed` を適用し、正の `2^127` を拒否し、負の `-2^127` を許可する。

source policy では、`to_i128_radix` に `str_slice` が戻らないこと、shared u128 digit parser が overflow check 前置を維持することを固定した。doctest には `"-"` と invalid radix の Err case を追加した。

同 checkpoint の native release RPN stage-only 測定。`after best` は同一変更後の最良 warm run、`after post-trunk` は `trunk build --release` 後に改めて実行した確認値である。

| stage | before | after best | after post-trunk |
|---|---:|---:|---:|
| `resource_initialized_i32_scalar_summaries` | 1256ms | 1172ms | 1450ms |
| `resource_initialized_raw_init_summaries` | 2549ms | 2239ms | 2647ms |
| `resource_initialized_function_checks` | 3139ms | 1767ms | 1965ms |
| `resource_initialized_moves` | - | 5236ms | 6139ms |
| `resource_static_check` | 7870ms | 6104ms | 7086ms |

この改善は stdlib 実装と path-state ownership の局所削減であり、初回 compile 0.5 秒未満にはまだ届かない。次の根本対応は、`ResourceSummaryCacheNamespaceKey` の下で function body hash、generic type-argument hash、source capability policy hash、summary kind/version を組み合わせ、arena 非依存の Resource summary stable mirror value だけを session cache へ保存することである。

## Resource static check inner timing checkpoint

2026-06-01 の checkpoint では、RPN same-session edit の残り時間を `resource_static_check` の内訳と function-level replay probe 数へ分解した。これは safety 判定を変える最適化ではなく、changed-function-only proof replay を入れる前の観測境界である。

追加した観測:

- `compile_stage_timings` に `resource_static_lowering`、`resource_static_lowering_coverage`、`resource_static_initialized_moves`、`resource_static_borrow_lifetimes`、`resource_static_effect_boundaries`、`resource_static_owner_obligations` などを追加する。
- `CompilerSession.loader_cache_stats_json()` に Resource static check の function/op 数を出す。
- raw alias、i32 scalar、raw-init、collection-slot、final initialized check について、replay probe / hit / miss の function 数を出す。

`tmp/rpn_static_inner_probe_20260601.json` の release Web RPN same-session unused local edit 測定では、base `compile_ms=10377`、edit `compile_ms=2384` だった。edit の主要 stage は `resource_static_initialized_moves=1056.579ms`、`resource_static_owner_obligations=878.709ms`、`resource_typecheck=171.733ms` である。

edit delta では `resource_summary_value_recomputed_ops=0` だが、raw alias は 288 関数 probe / 287 hit / 1 miss、i32 scalar は 209 probe / 209 hit、raw-init は 151 probe / 151 hit、final initialized check は 288 probe / 287 hit / 1 miss だった。したがって残り時間は stable mirror の false miss ではなく、未変更関数の proof replay probe と、owner obligations の全関数再検査による固定費である。

## Owner obligation pass cache checkpoint

2026-06-01 の checkpoint では、owner obligation の全関数再検査を diagnostic-free pass
cache として `ResourceSummaryValueCache` へ接続した。これは `OwnerReturnSummary` を保存する
cache ではなく、`ResourceOwnerCheckEngine::check_function` の結果が diagnostics を持たない
場合だけ、現在の compile でも pass と見なせることを保存する。

cache key は function body hash、dependency closure hash、source capability policy hash、
function-local type boundary、generic boundary、namespace を含む。diagnostics がある関数は
保存しない。cached pass では `final_owners` を materialize せず、`deferred` counter だけを
戻す。現在の compiler gate は owner diagnostics だけを失敗条件として消費するため、この
API は pass-only replay として閉じる。将来 `final_owners` を後続 stage が読む場合は、別の
stable owner state mirror を設計し、この fast path とは分ける。

`tmp/rpn_owner_obligation_cache_probe_final_20260601.json` の release Web RPN same-session string
literal edit 測定では、base `compile_ms=10801`、edit `compile_ms=3006` だった。edit delta
では `resource_owner_obligation_function_checks=0`、`resource_owner_obligation_function_check_ops=0`、
`resource_summary_value_owner_obligation_check_replay_hit_functions=288` になり、owner checker
本体は全関数で replay できている。

ただし edit の `resource_static_owner_obligations` はまだ `1534.075ms` 残る。これは cached
pass の前に `compute_owner_return_summaries` が全関数 fixed-point を走らせるためである。
`OwnerReturnSummary` は `TypeId`、projection、variant condition、host memory span、storage
origin marker などを含むため、単純な session-local `Vec<OwnerReturnSummary>` reuse は不可である。
次の根本対応は `ISS-20260601T174500000Z-OWNER-RETURN-SUMMARY-NEEDS-STABLE-CACH-8A7B61D2` として、
owner return summary の stable mirror value cache を設計する。

## Final initialized pass plan checkpoint

2026-06-01 の checkpoint では、final initialized function check の pass-only replay を
changed-function-aware にした。これは full `ResourceFunctionCheck` の永続化ではなく、
diagnostic-free / auto-drop-free pass として既に安全に保存できた関数について、次回 compile で
dependency closure hash の再構築と replay probe を省くための compile-local plan である。

plan は前回 compile の local fingerprint と現在 compile の local fingerprint を比較する。
fingerprint は関数 identity、Resource IR body hash、type boundary、generic boundary、source
capability policy hash から成り、`TypeId`、`Span`、`SourceMap`、final cell state を含まない。
変化した関数から `ResourceSummaryDependencyGraph` の reverse dependents を辿り、affected set
を作る。affected ではない関数だけ、前回 snapshot の `ResourceCheckDeferred` を checked pass
として戻す。

この checkpoint の安全境界:

- namespace、関数 order、fingerprint の構築に失敗した場合は conservative-all に倒す。
- diagnostic または auto-drop point を持つ関数は snapshot pass にしない。
- summary fixed-point 側の raw alias / i32 scalar / raw-init preseed loop にはまだ適用しない。
  これらは affected 関数の再計算に callee summary materialization が必要であり、単純に全関数
  probe を消すと caller summary が空に見える危険がある。

Web / Node の cache stats には
`resource_summary_value_initialized_function_check_plan_skip_functions` と
`resource_summary_value_initialized_function_check_plan_skip_ops` を追加した。これにより、通常の
`resource_summary_value_initialized_function_check_replay_probe_functions` が減った場合でも、
checked pass として省略された関数数を確認できる。

`tmp/rpn_final_initialized_pass_plan_20260601.json` では、release Web RPN same-session string
literal edit が base `compile_ms=9998`、edit `compile_ms=2178` だった。edit delta は
`resource_summary_value_initialized_function_check_plan_skip_functions=288`、
`resource_summary_value_initialized_function_check_plan_skip_ops=3639`、
`resource_summary_value_initialized_function_check_replay_probe_functions=0` である。final check
の replay probe 固定費は消えたが、total `compile_ms` はまだ 0.1 秒以下ではないため、次は
summary fixed-point 側と typed expression subtree query を継続する。

## CompilerSession first checkpoint

2026-05-27 の first checkpoint では、公開 API の境界を先に session 化した。現在の `CompilerSession` は bundled stdlib source table の保持までを行い、parse / typecheck / Resource IR summary cache はまだ持たない。

この段階での目的は、Web / Node 側の呼び出し元を「1 compile call = stateless function」から「WASM instance 内の session」へ移すことである。これにより、後続の parsed stdlib module、public signature table、Resource IR summary template を同じ API に追加できる。

追加測定:

| case | command / artifact | result |
|---|---|---|
| Web release session minimal smoke | `nodesrc/run_test.js` default warmup | `compiler_session=true`、`compile_ms=3`、`wasm_call_ms=3`、`stdlib_vfs_mode=bundled` |
| Web release session aggregate smoke | `tmp/perf_alloc_probe.nepl` + default warmup | `compiler_session=true`、`compile_ms=16`、`wasm_call_ms=16`、`stdlib_vfs_mode=bundled` |
| Web release session cold minimal | `NEPL_RUN_TEST_SKIP_COMPILER_WARMUP=1` + `tmp/minimal_perf.nepl` | `compiler_session=true`、`compile_ms=160`、`wasm_call_ms=113`、`stdlib_vfs_mode=bundled` |

`CompilerSession` がまだ semantic cache を持たないため、aggregate case は 10ms 未満には届いていない。次の checkpoint では、stdlib parse/import/type arity/typecheck artifact を session に持たせる。

stdlib freshness は、同一 timestamp や process-local mtime cache の stale 判定を避けるため、artifact に埋め込まれた `fnv1a64` content hash と local stdlib tree hash の比較を優先する。旧 artifact で hash API が存在しない場合だけ mtime fallback を使う。

Web playground の terminal worker と tutorial runtime も同じ session API 境界へ寄せた。これにより、playground 側でも通常の compile request は `CompilerSession` の `compile_outputs_with_vfs` / `compile_source_with_vfs_and_profile` を通る。旧 artifact 互換のため stateless API と full stdlib VFS fallback は残すが、fallback は該当 session method が存在しない場合だけに限定する。

workspace VFS には explorer 表示用の read-only stdlib files と、実行時に参照する data files / binary outputs が入るが、compile overlay として必要なのは user-editable `.nepl` source module だけである。`VFS.serializeForCompile()` は read-only file、`.nepl` 以外の runtime data file、binary output を除外し、entry source と editable user module だけを worker へ渡す。WASI 実行時は `runtimeVfsData` として full VFS snapshot を別に渡すため、compile 軽量化で実行時ファイルは失われない。bundled stdlib は session 側の source table が担当する。

Web terminal は compile 用 Worker を artifact URL 単位で保持する。`neplg2 build` の連続実行は同じ Worker / WASM instance / `CompilerSession` を再利用し、次段階の parsed stdlib cache をそのまま載せられる。`neplg2 run` は compile だけを persistent Worker に通し、生成された wasm の実行は一回限りの runtime Worker に渡す。これにより CompilerSession の寿命と WASI process の寿命を分け、stdin / runtime trap / VFS side effect が compile cache に混ざらない。

## LoaderSessionCache checkpoint

2026-05-27 の second checkpoint では、`CompilerSession` に loader-level parsed stdlib cache を接続した。これは stdlib prechecked artifact の最初の実装単位であり、まだ full typed HIR cache ではない。

cache する artifact:

- bundled stdlib の raw parsed `Module`。
- parsed module から得た `SourceCapabilities`。
- cache hit / miss / store / bypass の統計。

cache しない artifact:

- `SourceMap`。
- merged module 全体。
- `ImportResolution`。
- typechecked HIR。
- `TypeCtx` / `TypeId`。
- Resource IR summary。
- codegen fragment。

この境界にした理由は、`SourceMap` が compile ごとに append-only の `FileId` を割り当てるためである。AST / HIR / diagnostics / source capability proof には `Span` が含まれるので、古い `FileId` をそのまま再利用すると、別 source へ capability を与える、import visibility がずれる、診断位置が誤る、という壊れ方をする。

実装では、cache に保存する AST を `CACHED_MODULE_FILE_ID` へ正規化し、cache hit 時に現在 compile の `SourceMap` が割り当てた `FileId` へ再投影する。`Span::dummy()` は実 source 位置ではないため再投影しない。source capability は byte range と capability kind だけを持つので、同じ source hash に対してのみ再利用し、現在の `SourceMap` file slot に設定する。

`CompilerSession.loader_cache_stats_json()` は Node / Web から cache hit を観測するための API である。`nodesrc/run_test.js` はこの統計を `timing.compiler_session_cache_before` / `timing.compiler_session_cache_after` として JSON output へ含める。

subagent review 後に、次の safety regression を追加した。

- 同一 canonical stdlib path でも source hash が変わる場合は cache hit しない。
- imported type arity hints が変わる場合は、source text が同じ dependent module でも cache hit しない。
- forced stdlib VFS / stdlib overlay path は session API 経由でも bundled stdlib parsed module cache を使わない。
- `LoaderSessionCache` は `new(namespace_hash)` で作成し、空 namespace の `Default` cache を公開しない。

追加測定:

| case | command / artifact | result |
|---|---|---|
| Web release session minimal after warmup | `tmp/minimal_perf.nepl` + `nodesrc/run_test.js` | `compiler_session=true`、`compile_ms=2`、`wasm_call_ms=2`、`cache 4 hit / 4 store` |
| Web release session aggregate first after warmup | `tmp/perf_alloc_probe.nepl` + `nodesrc/run_test.js` | `compiler_session=true`、`compile_ms=19`、`wasm_call_ms=19`、`cache 4 hit + 1 miss/store` |
| Web release session aggregate second same process | same preloaded runner / same `CompilerSession` | `compiler_session=true`、`compile_ms=3`、`wasm_call_ms=3`、`cache hits 4 -> 9` |

同一 session で同じ aggregate source を再compileした場合は、stdlib parsed module cache が全て hit し、10ms 未満に入った。一方で、まだ初回に import されていない stdlib module を含む aggregate case では、追加 module の parse / import / typecheck / Resource IR / codegen が残るため 10ms を超えることがある。次 checkpoint は public surface / import graph / Resource IR summary cache へ進める。

## Source arity surface checkpoint

2026-05-27 の third checkpoint では、loader-level の source arity surface cache を `LoaderSessionCache` に追加した。これは typed public signature table ではなく、NEPLg2.1 prefix type parser が依存 module の型arity境界を知るための未型付け artifact である。

cache する artifact:

- source 内で宣言された `struct` / `enum` / `trait` の local type arity hints。
- `#prelude`、`#import`、`#include` から得られる type arity preload path。
- shallow cycle recovery が見る `#import pub` と `#include` の public re-export path。
- root source だけに適用する default prelude の resolved path と `#no_prelude` 判定。
- cache hit / miss / store の統計。

cache しない artifact:

- 依存先 module を再帰的に畳み込んだ type arity result。
- `SourceMap` / `FileId` / `Span`。
- `ImportResolution`。
- typed signature / typed HIR。
- `TypeCtx` / `TypeId`。
- Resource IR summary。

依存先の public type arity を親 module の surface value に畳み込まない理由は、依存先 source が変わったときに親 source hash が変わらなくても parser boundary が変わり得るためである。親 surface は public re-export edge だけを保持し、依存先 surface は依存先の source hash で別 query として引き直す。これにより、facade source が unchanged で cache hit しても、re-export 先の `Box<.T>` が `Box<.T,.U>` に変わった場合は新しい arity が使われる。

`CompilerSession.loader_cache_stats_json()` は `arity_surface_hits` / `arity_surface_misses` / `arity_surface_stores` を返す。Node / Web 側の timing JSON は既存の `compiler_session_cache_before` / `compiler_session_cache_after` 経由でこの統計を観測できる。

追加 regression:

- source text が同じ場合、source arity surface が session 内で再利用される。
- 同じ cached surface から root / non-root の preload path を計算しても、default prelude は root にだけ入る。
- lexer error のある source では、旧 `type_arity_preload_paths` と同じく default prelude を preload せず、通常 parser path の lexer diagnostic を優先する。
- public re-export facade が cache hit しても、re-export 先 source hash の変化は新しい type arity として反映される。

追加測定:

| case | command / artifact | result |
|---|---|---|
| Web release session minimal after arity surface cache | `tmp/minimal_perf.nepl` + same preloaded `CompilerSession` | `compile_ms=2`、`wasm_call_ms=2`、parsed cache `4 hit / 4 store`、arity surface `4 hit / 6 store` |
| Web release session aggregate first after arity surface cache | `tmp/perf_alloc_probe.nepl` + same preloaded `CompilerSession` | `compile_ms=15`、`wasm_call_ms=15`、parsed cache `8 hit / 5 store`、arity surface `8 hit / 8 store` |
| Web release session aggregate second after arity surface cache | same source / same `CompilerSession` | `compile_ms=4`、`wasm_call_ms=4`、parsed cache hits `8 -> 13`、arity surface hits `8 -> 14` |

## Source-directed loader prewarm checkpoint

2026-05-27 の fourth checkpoint では、Node / Web の warm session が root source の import surface から到達する bundled stdlib roots を compile 前に loader cache へ入れるようにした。subagent review では、全 `bundled_sources.keys()` を総なめする案は、program dependency graph ではなく packaging file list に依存し、未使用 module の parse failure や将来の invalidation boundary を不明確にするため却下した。

prewarm 対象:

- root source の default prelude。
- 明示 `#prelude`。
- root source に直接書かれた `#import`。
- root source に直接書かれた `#include`。
- 上記 roots から通常 loader が辿る configured stdlib dependency closure。

prewarm しない対象:

- bundled stdlib の全ファイル一覧。
- user source と user VFS module の arity surface。
- forced stdlib VFS / local stdlib override / `/stdlib` overlay がある compile。
- typed HIR、`ImportResolution`、Resource IR、codegen fragment。

この checkpoint で、`LoaderSessionCache` の arity surface cache は configured stdlib path に限定した。従来の実装では user entry source の arity surface も source hash key で保存され得たため、`FileId` 安全性は壊れないものの、長寿命 `CompilerSession` の memory boundary と「stdlib artifact cache」という設計コメントが一致していなかった。現在は user source scan を `arity_surface_bypasses` として観測し、cache value には残さない。

`CompilerSession.prewarm_loader_cache_for_source(entry_path, source)` は source-directed prewarm だけを公開する。Node runner は `selectStdlibVfsMode(meta) == "bundled"` の場合だけ呼び、forced / fs override では skip reason を timing へ残す。prewarm 中の loader error は optimization failure として `compiler_session_prewarm_error` に記録し、通常 compile path を続行する。これにより、prewarm 専用の失敗が本来の compile diagnostic を隠さない。

追加 regression:

- user source の arity surface は `LoaderSessionCache` に store/hit されず、bypass として観測される。
- source-directed prewarm は root source から到達する stdlib root だけを数え、依存 closure の parsed module / arity surface を cache する。
- forced stdlib VFS では bundled stdlib prewarm を呼ばない。
- prewarm error が起きても、run result の compile error は本来の compile path の error を保持する。

追加測定:

| case | command / artifact | result |
|---|---|---|
| Web release warmup prewarm | `createRunner("web/dist")` | `prewarm_count=1`、`prewarm_ms=18`、`warmup_ms=405`、bundled file count `573` |
| Web release source-prewarm minimal | `tmp/minimal_perf.nepl` + same `CompilerSession` | `compile_ms=3`、`prewarm_ms=1`、`wasm_call_ms=2`、`prewarm_count=1` |
| Web release source-prewarm aggregate first | `tmp/perf_alloc_probe.nepl` + same `CompilerSession` | `compile_ms=15`、`prewarm_ms=3`、`wasm_call_ms=11-12`、`prewarm_count=2` |
| Web release source-prewarm aggregate second | same source / same `CompilerSession` | `compile_ms=4-5`、`prewarm_ms=1`、`wasm_call_ms=3-4`、`prewarm_count=2` |

この checkpoint は cache boundary と観測性を改善したが、aggregate first の total `compile_ms` はまだ 10ms 未満へ固定できていない。prewarm は loader-level query だけなので、初回 aggregate には typecheck / typed public surface / Resource IR / codegen の未cache work が残る。次 checkpoint は、typed public surface に進む前に logical import graph と dependency public surface hash の安定表現を実装し、stdlib facade / re-export 変更時の invalidation を明確にする。

## Private memoization purity design checkpoint

2026-05-31 に、compiler cache の将来設計と同じ純粋 query model を source language 側の `memo_call` へ展開するため、[NEPLg2 private effect / memoization purity design](./private_effect_memoization_purity_design.md) を追加した。

この設計では、`Pure` を「内部 mutation がない」ではなく「外部観測可能な effect がない」と定義する。`PrivateCache` / `PrivateState` はそれ自体を `Pure` と同一視せず、fresh private region が return value、public state、raw pointer、reference、owner token、function identity へ escape しない場合だけ `Pure` へ mask する。

短期の `memo_call` は compiler-known trusted primitive とし、non-capturing named pure function value と Copy 相当の `MemoKey` / `MemoValue` だけを対象にする。これは runtime feature の設計だが、self-host compiler の query cache / incremental compile でも「純粋関数の結果を private cache に保存し、外部観測上は pure」と表現するための前提になる。

同じ review で、RPN compile の現在の支配点は Resource summary value cache の raw-init replay と再確認した。`tmp/rpn_owner_boundary_20260531.json` の初回測定では `compile_ms=9615`、`raw_init_param_facts_stores=165`、`bypasses=60`、`incomplete_leaf=37`、`reprojection_value=23`、`param_cell_stable_type=23` である。次の性能 checkpoint は、`reprojection_context=0`、`unstable_key=0`、`unstable_entry=0` を維持しながら、labelled open generic の provenance / ordinal を stable entry と key へ加えて `param_cell_stable_type` を減らす。

## Final initialized function check cache checkpoint

2026-05-31 の checkpoint では、raw-init / i32 scalar summary replay 後にも残っていた final initialized function check の全関数再実行を削るため、`ResourceFunctionCheck` の diagnostic-free stable entry cache を追加した。

cache する artifact:

- `final_cells`。
- `final_collection_slots`。
- `ResourceCheckDeferred`。

cache しない artifact:

- `ResourceCheckDiagnostic`。diagnostic は `Span` を持つため、古い source map 由来の診断を replay しない。
- `auto_drop_points`。drop elaboration が span 付きの drop plan として後続生成に使うため、span-free drop plan と current body への再束縛を設計するまでは no-store にする。
- `PlaceRoot::Unknown` を含む final state。

key は Resource IR body hash、source capability policy hash、typed signature/type boundary、generic type argument mode、dependency closure hash を含む。final state に含まれる `TypeId` は `ResourceSummaryStableTypeKey` へ変換し、replay 時に現在 compile の `TypeCtx` へ戻せる場合だけ hit とする。

`ResourceId` / `StorageId` は stable value へ直接保存しない。関数本文を決定的順序で走査し、temporary / storage root を出現順 ordinal に正規化して保存する。replay 側でも現在の同じ body から ordinal を実 id へ戻すため、compile session 内の id 割当が変わっても stale id を直接使わない。

focused regression:

- 同一 diagnostic-free function は二回目 compile で `ResourceCheckEngine` を再実行しない。
- function body が変わると miss になり通常 checker へ戻る。
- `auto_drop_points` を持つ function check は no-store になる。

release Web RPN same-session code edit 測定 `tmp/rpn_final_check_cache_code_edit_20260531.json` では、base `compile_ms=12465`、unused local 追加 edit `compile_ms=8254` だった。edit delta は `resource_initialized_function_checks=128`、`resource_initialized_function_check_ops=2202`、`resource_summary_value_initialized_function_check_hits=160`、`resource_summary_value_replayed_ops=2122` である。

直前の i32 scalar checkpoint では code edit delta が `resource_initialized_function_checks=288` だったため、final check cache は一部の stdlib function check を skip できている。一方で、まだ `initialized_function_check_reprojection_value_type_bypasses=73` と `initialized_function_check_reprojection_value_place_bypasses=52` が残る。これは final check cache 自体を閉じるには不十分なので、残件は `ISS-20260531T061756145Z-FINAL-INITIALIZED-CHECK-REPLAY-STILL-5CB1018A` で追跡する。

## Final initialized function check reprojection checkpoint

2026-05-31 の final initialized function check checkpoint では、RPN code edit で残っていた replay bypass のうち、body-local type boundary と function-local storage offset place を root cause とするものを解消した。

変更した stable replay boundary:

- `ResourceSummaryTypeReprojection::new_for_initialized_function_check` を追加し、function body に現れる type を final check replay 専用 boundary へ入れる。
- final check entry の storage offset place は、parameter-relative place だけでなく function-local `Temporary` / `Storage` ordinal を持つ resource place として保存できる。
- Resource IR body hash が同一である replay では、generic や raw storage view のため現在 `TypeCtx` だけで layout を再計算できない projection suffix を、保存済み stable projection surface から戻す。ただし offset 内 place や type 再投影に失敗した場合は従来通り通常 checker へ戻る。
- summary/raw-init 系の `SummaryOffset` 再投影は、新しい resource-place offset を受け取らない。これは final check entry 専用 surface を他の summary kind へ混ぜないためである。

測定:

| case | before | after |
|---|---:|---:|
| RPN code edit `compile_ms` | 8254ms | 6021ms |
| `resource_initialized_function_checks` | 128 | 20 |
| `resource_initialized_function_check_ops` | 2202 | 371 |
| `resource_summary_value_initialized_function_check_hits` | 160 | 268 |
| `initialized_function_check_reprojection_value_place_bypasses` | 52 | 0 |
| `initialized_function_check_reprojection_value_type_bypasses` | 73 | 7 |

測定 JSON は `tmp/rpn_final_check_reprojection_boundary_20260531.json` に保存した。今回の修正で final check の主要な replay false miss は解消したが、7 件の type-only bypass は `ISS-20260531T065418483Z-FINAL-INITIALIZED-CHECK-RESIDUAL-TYP-320256A9` に分離した。さらに RPN code edit は `resource_raw_alias_summary_recomputations=288`、`resource_raw_init_summary_recomputations=81` も残しており、次の支配項は raw alias / raw-init side の incremental replay である。

## Final initialized function check residual type checkpoint

2026-05-31 の residual type checkpoint では、残った 7 件の final check type-only replay bypass を細分した。追加 counter は `place_type`、`projection_result_type`、`cell_state_type`、`collection_slot_state_type` を分ける。

`tmp/rpn_final_check_residual_type_counter_20260531.json` では、7 件すべてが `projection_result_type` だった。これは stable place type 自体が戻らない問題ではなく、base type + projection suffix から現在 `TypeCtx` で再計算した型と、final checker が保存した place type の canonicalization が一致しない問題である。

final check entry は Resource IR body hash と stable place surface が同一の場合にだけ replay される。保存済み place type が current boundary へ再投影できている場合、その型は Resource IR final state の proof surface として扱えるため、projection layout の再計算型が一致しない場合も保存済み型を採用する。TypeCtx 全体から似た型を探す緩和は導入しない。

測定:

| case | before | after |
|---|---:|---:|
| RPN code edit `compile_ms` | 6021ms | 5770ms |
| `resource_initialized_function_checks` | 20 | 13 |
| `resource_initialized_function_check_ops` | 371 | 263 |
| `resource_summary_value_initialized_function_check_hits` | 268 | 275 |
| `initialized_function_check_reprojection_value_type_bypasses` | 7 | 0 |

測定 JSON は `tmp/rpn_final_check_residual_type_fix_20260531.json` に保存した。final check replay の type/place false miss は解消したため、次の支配項は `resource_raw_alias_summary_recomputations=288` と `resource_raw_init_summary_recomputations=81` である。これらは `ISS-20260531T071945698Z-RAW-ALIAS-SUMMARIES-NEED-STABLE-MIRR-4DCE44A8` と `ISS-20260531T071956084Z-RAW-INIT-RESIDUAL-RECOMPUTATIONS-NEE-C36FBACE` に分離した。

## Raw alias summary stable mirror checkpoint

2026-05-31 の raw alias checkpoint では、`RawCellAddressReturnSummary` を stable entry として保存し、現在 compile の function signature / projection / type boundary へ再投影できる場合だけ raw alias fixed-point worklist を preseed するようにした。cache key には function body hash、source capability policy hash、dependency closure hash、summary type boundary、generic argument mode を含める。stable value には `TypeId`、`Span`、`SourceMap`、compile ごとの raw address graph state を入れない。

empty summary も entry として保存する。raw alias summary vector 上は「summary なし」と同じ意味だが、worklist へ戻す必要がない no-alias 関数を区別できるため、同じ `CompilerSession` 内の微小編集で fixed-point の初期 pending を減らせる。preseed で初期 pending から外した関数も relevant には残すため、依存先 summary が変わった場合は `notify_changed` により再度 worklist へ入る。

測定:

| case | before | after |
|---|---:|---:|
| RPN code edit `compile_ms` | 5770ms | 7142ms |
| `resource_raw_alias_summary_recomputations` | 288 | 38 |
| `resource_summary_value_raw_alias_return_entry_hits` | 0 | 65 |
| `resource_summary_value_raw_alias_return_entry_stores` | 0 | 73 |
| `resource_summary_value_raw_alias_return_entry_reprojection_value_bypasses` | n/a | 13 |
| `resource_summary_value_raw_alias_return_entry_unstable_key_bypasses` | n/a | 0 |
| `resource_summary_value_raw_alias_return_entry_unstable_entry_bypasses` | n/a | 0 |

測定 JSON は `tmp/rpn_raw_alias_cache_code_edit_20260531.json` に保存した。raw alias fixed-point の全関数規模再計算は解消したが、全体 compile time はまだ秒単位であり、同じ測定の edit delta では `resource_raw_init_summary_recomputations=81`、`resource_initialized_function_checks=13` も残る。残った raw alias 側の 13 件の `reprojection_value` bypass は `ISS-20260531T075621000Z-RAW-ALIAS-RESIDUAL-REPROJECTION-VAL-9A5D0C3E` に分離した。

## RPN code-edit stage breakdown checkpoint

2026-05-31 の complete raw-init leaf replay と return / byte-range type canonicalization 後、RPN same-session code edit は raw-init replay miss をほぼ解消したが、まだ秒単位である。`tmp/rpn_return_type_canonicalization_code_edit_20260531.json` では base `compile_ms=8861`、local `i32` binding 追加 edit `compile_ms=6703`、edit delta は `resource_summary_value_replayed_ops=253`、`resource_summary_value_recomputed_ops=21`、`raw_init_param_facts_bypasses=0` だった。

Native release CLI で同じ RPN workload を stage timing した結果、`resource_static_check=6950ms` の大半は `resource_initialized_moves=6050ms` であり、そこに `resource_initialized_raw_init_summaries=2502ms`、`resource_initialized_i32_scalar_summaries=1558ms`、`resource_initialized_function_checks=1875ms` が含まれている。raw-init summary replay は有効になっているが、i32 scalar summary と final initialized function check はまだ compile ごとに全関数規模で走っている。

この checkpoint では `CompilerSession.loader_cache_stats_json()` に次の counter を追加する。

- `resource_raw_alias_summary_recomputations` / `resource_raw_alias_summary_count`
- `resource_i32_scalar_summary_recomputations` / `resource_i32_scalar_summary_count`
- `resource_raw_init_summary_recomputations` / `resource_raw_init_summary_count`
- `resource_collection_slot_summary_recomputations` / `resource_collection_slot_summary_count`
- `resource_initialized_function_checks` / `resource_initialized_function_check_ops`

これらは semantic cache hit/miss ではなく、raw-init replay 後に残る fixed-point / final check の実行量を Web / Node の same-session JSON で観測するための補助統計である。通常の安全性判定、source capability、Resource IR proof には影響しない。次の根本対応は、i32 scalar summary の stable mirror / replay と、final initialized function check の stdlib prechecked artifact または function-level stable result cache に分けて進める。

`tmp/rpn_stage_breakdown_code_edit_20260531.json` の first measurement では、unused local 追加 edit が `compile_ms=6771` で、delta は `resource_i32_scalar_summary_recomputations=209`、`resource_raw_init_summary_recomputations=81`、`resource_initialized_function_checks=288`、`resource_initialized_function_check_ops=3642`、`resource_summary_value_replayed_ops=253`、`resource_summary_value_recomputed_ops=21` だった。これにより、raw-init replay をさらに詰めるだけでは不十分で、i32 scalar summary replay と final function check replay を別 issue として進める必要が確定した。

2026-05-31 の i32 scalar stable mirror checkpoint では、`I32ScalarReturnFacts` の aliases / offsets / relations / constants / return conditions / parameter conditions を `ResourceSummaryValueCache` の stable entry として保存し、`TypeId` を現在 compile の function signature へ再投影できる場合だけ worklist 前に preseed するようにした。i32 scalar summary は callee summary と raw-alias summary を取り込むため、key には function body だけでなく dependency closure の body / source capability policy / type boundary hash も含める。facts が空の relevant function も空 summary として cache し、no-fact function が微小編集ごとに worklist へ戻る固定費を消す。

`tmp/rpn_i32_scalar_empty_cache_code_edit_20260531.json` では、RPN same-session code edit の delta が `resource_i32_scalar_summary_recomputations=14` まで減った。`resource_summary_value_i32_scalar_return_facts_hits=429`、`resource_summary_value_replayed_ops=682` で、i32 scalar facts と raw-init facts の replay が同じ session cache 上で効いている。一方で edit compile は `compile_ms=6496` でまだ秒単位であり、次の支配項は `resource_raw_init_summary_recomputations=81` と `resource_initialized_function_checks=288` である。

## Source import surface checkpoint

2026-05-27 の fifth checkpoint では、logical import graph を `ImportResolution` の置き換えとしていきなり導入せず、まず loader の未型付け source surface を import edge 表現へ広げた。subagent review では、`ImportResolution` が `FileId` に依存すること、typed public surface hash に `TypeId` や mangled symbol をそのまま使うと compile ごとの arena / `Span` に依存することが指摘された。そのため、この checkpoint では `FileId` / `Span` / `ImportResolution` / typed HIR / `TypeId` を cache value に入れない境界を維持する。

変更した境界:

- parser に `parse_import_directive_parts` を追加し、loader の raw `#import` text parsing と parser の import clause parsing が分岐しないようにした。
- `CachedAritySurface` の内部表現を path list から `SourceImportEdge` list へ変更した。
- edge value は kind (`Prelude` / `Import` / `Include`)、resolved target path、visibility、import clause、source order を持つ。
- `type_arity_preload_paths` と shallow public re-export recovery は、この edge list から従来と同じ path list を派生する。
- root-only default prelude、lexer error 時の no-preload、stdlib-only long-lived cache boundary は維持する。

この checkpoint は observational な下準備であり、merged module 作成、typecheck、Resource IR、codegen の入力は変えない。今後の logical import graph は、この source import surface から path/source-hash keyed な nodes / edges / reverse edges / public re-export edges を作り、compile ごとの `SourceMap` へ materialize して `ImportResolution` を段階的に置き換える。

追加 regression:

- import surface が source order を保ったまま preload path を派生する。
- public import と include が同じ surface から public re-export path として派生する。
- `#import pub "types" as { Box as PublicBox, Result::* }` の visibility と selective import clause が path-only edge へ潰れない。

追加測定:

| case | command / artifact | result |
|---|---|---|
| Web release source-import-surface minimal | `tmp/minimal_perf.nepl` + same `CompilerSession` | `compile_ms=2`、`prewarm_ms=0`、`wasm_call_ms=2` |
| Web release source-import-surface aggregate first | `tmp/perf_alloc_probe.nepl` + same `CompilerSession` | `compile_ms=16`、`prewarm_ms=3`、`wasm_call_ms=13` |
| Web release source-import-surface aggregate second | same source / same `CompilerSession` | `compile_ms=4`、`prewarm_ms=1`、`wasm_call_ms=3` |

## Root import surface prewarm guard checkpoint

2026-05-27 の sixth checkpoint では、source-directed prewarm を semantic cache と混同せず、同じ root import surface に対する loader prewarm の重複実行だけを session 内で省略する guard として実装した。

追加した境界:

- `nepl-core::loader` は root source から prewarm surface hash と prewarm roots を同時に計算する。
- hash には loader cache version、canonical stdlib root、root default prelude state、`#no_prelude`、lexer error outcome、prelude/import/include edge の kind / resolved target path / visibility / import clause / source order を含める。
- hash には root source body、コメント、空白、local type arity hints、`FileId`、`Span`、`ImportResolution`、typed HIR、`TypeId`、Resource IR、codegen fragment を含めない。
- `CompilerSession` は prewarm 成功後に surface hash と前回の warmed root count だけを保持する。user source の AST / surface value / token stream は保持しない。
- 同じ surface の再 prewarm は前回の warmed root count を返し、provider traversal と stdlib loader query traversal は実行しない。no-op だった事実は `prewarm_surface_hits` / `prewarm_surface_stores` で観測する。
- prewarm failure では surface hash を記録しないため、同じ source を次回再試行できる。
- `clear_loader_cache()` は parsed module / arity surface cache と同時に prewarmed surface map と hit/store counter を消す。
- forced stdlib VFS、FS stdlib override、compile VFS 内の `/stdlib` overlay では bundled stdlib prewarm を呼ばない。

追加 regression:

- body-only edit では root prewarm surface hash と roots が変わらない。
- import path、import clause、relative import の解決先、lexer error outcome は hash に反映される。
- `provider_session_cache_can_prewarm_stdlib_loader_queries` の user-source arity bypass 観測を維持する。
- Node runner は同じ session の 2 回目 prewarm reuse を `prewarm_surface_hits` で観測する。
- Node runner は `/stdlib` overlay がある compile request では bundled prewarm を skip する。

追加測定:

| case | command / artifact | result |
|---|---|---|
| Web release aggregate first after prewarm guard | `tmp/perf_alloc_probe.nepl` + same `CompilerSession` | `compile_ms=16`、`prewarm_ms=3`、`prewarm_count=2`、`wasm_call_ms=13` |
| Web release aggregate body-only edit | return literalだけを変更 + same `CompilerSession` | `compile_ms=3`、`prewarm_ms=0`、`prewarm_count=2`、`prewarm_surface_hits=1`、`wasm_call_ms=3` |

この checkpoint で微小変更時の loader prewarm は 10ms budget からほぼ外れた。一方で aggregate first は `wasm_call_ms=13` が支配的であり、初回に未cache typed public surface / Resource IR summary / codegen が残る。したがって次段階は、logical import graph と dependency public surface hash を安定化し、typed public surface cache へ進む。

## Public surface hash checkpoint

2026-05-27 の seventh checkpoint では、typed public surface cache に進む前の未型付け artifact として、stdlib parsed module cache に `public_surface_hash` を追加した。これは downstream module に見える signature / lookup context の変化を観測するための staging であり、この hash だけで typed HIR、`TypeId`、Resource IR summary、codegen fragment を再利用しない。

hash に含めるもの:

- loader cache version。
- source import surface から得られる prelude / import / include edge の kind、resolved target path、visibility、import clause、source order。
- `#no_prelude`、implicit default prelude、default prelude path。
- public function signature、generic type parameter bounds、effect、arity、`noshadow`。
- public function alias の alias 名、target 名、`noshadow`、同一 module 内 target callable signature。
- public struct / enum / trait の header、trait capability、trait method signature。
- impl header と method signature。trait / inherent impl lookup に影響し得るため、public filter はまだ掛けない。
- public extern signature、public re-export directive。dependency surface が渡されない単体 test では raw import/include/prelude directive も conservative に含める。

hash に含めないもの:

- docs / comments / whitespace。
- private function body、public function body、raw wasm / raw llvm body。
- `FileId`、`Span`、`SourceMap`、`ImportResolution`。
- typed HIR、`TypeCtx` / `TypeId`、mangled symbol、Resource IR、codegen fragment。

subagent review では、public signature text が同じでも private import / prelude / include の解決先が変われば typed public surface が変わり得ること、public alias が private helper を公開する場合は helper signature を key に含める必要があること、`noshadow` は cross-file binding behavior の一部であることが指摘された。今回の実装ではこの範囲を loader artifact に反映した。この時点では dependency aggregate hash、reverse graph、canonical typed signature table が未実装だったため、次 checkpoint で `SourceImportEdge` と module `public_surface_hash` を畳み込む query を `LoaderSessionCache` とは別の semantic cache 境界へ接続する方針にした。

追加 regression:

- public function body と private helper の非alias signature change では hash が変わらない。
- public function signature と public re-export clause では hash が変わる。
- public alias が指す local callable signature では hash が変わり、target body-only edit では変わらない。
- private import edge の resolved target path / clause は、public signature text が同じでも hash に反映される。
- public `noshadow` の有無は hash に反映される。
- provider prewarm 後の同一 session load で `public_surface_hash_hits` が増える。

追加測定:

| case | command / artifact | result |
|---|---|---|
| Web release public-surface aggregate first | `tmp/perf_alloc_probe.nepl` + same `CompilerSession` | `compile_ms=17`、`prewarm_ms=3`、`wasm_call_ms=14`、`public_surface_hash_hits=13`、`public_surface_hash_stores=5` |
| Web release public-surface aggregate second | same source / same `CompilerSession` | `compile_ms=3`、`prewarm_ms=0`、`wasm_call_ms=3`、`public_surface_hash_hits=18` |
| Web release public-surface body-only edit | return literalだけを変更 + same `CompilerSession` | `compile_ms=3`、`prewarm_ms=0`、`wasm_call_ms=3`、`public_surface_hash_hits=23` |

この checkpoint は correctness-first の staging であり、aggregate first の compile time を直接下げるものではない。Zenn 方針の「純粋性や責務分割を活かした探索空間削減」に沿って、次の typed public surface cache が body-only edit を安全に再利用し、public dependency change では確実に invalidation できる根拠を用意した。

## Dependency aggregate public surface checkpoint

2026-05-27 の eighth checkpoint では、module 単体の `public_surface_hash` を、root source から到達する configured stdlib dependency closure の aggregate hash へ畳み込む query を追加した。これは typed public signature table ではなく、typed cache key の入力にするための loader-level staging artifact である。

追加した境界:

- `Loader::root_dependency_aggregate_public_surface_hash_for_source_with_cache` は root source の import surface と、到達する stdlib dependency entry `(canonical path, dependency aggregate hash)` を順序付きで畳み込む。
- stdlib module の dependency aggregate hash は、module 自身の `module_public_surface_hash`、children aggregate hash、ordered dependency entries を含む。
- aggregate cache key は canonical stdlib root、canonical module path、module public surface hash、child dependency aggregate hash であり、source body hash ではない。これにより body-only edit は parsed module cache miss になっても aggregate public surface cache は再利用できる。
- non-stdlib dependency edge は bundled stdlib aggregate cache の対象外なので、provider で読まず conservative external hash にして `dependency_aggregate_public_surface_hash_bypasses` で観測する。
- import cycle に戻る edge は source hash を含む conservative cycle hash にする。body-only edit でも過剰 invalidation し得るが、stale typed cache を作らないことを優先する。
- `CompilerSession.prewarm_loader_cache_for_source` は source-directed prewarm だけを行う。dependency aggregate public surface hash は通常の bundled-stdlib compile path で `ResourceSummaryCacheNamespaceKey` の入力として消費するが、Web playground の compile 前 prewarm hot path では計算しない。
- `CompilerSession.loader_cache_stats_json()` は `dependency_aggregate_public_surface_hash_hits` / `misses` / `stores` / `bypasses` を返す。

追加 regression:

- re-exported stdlib dependency の public body-only edit では root dependency aggregate hash が変わらず、aggregate cache hit が増える。
- re-exported stdlib dependency の public signature edit では root dependency aggregate hash が変わる。
- root source からの non-stdlib relative import は bundled stdlib aggregate cache で provider read されず、bypass として観測される。
- Node runner の session stats stub は dependency aggregate counter を timing JSON へ通す。

追加測定:

| case | command / artifact | result |
|---|---|---|
| Web release dependency-aggregate first | `tmp/perf_alloc_probe.nepl` + same `CompilerSession` | `compile_ms=17`、`prewarm_ms=3`、`wasm_call_ms=14`、`dependency_aggregate_public_surface_hash_hits=4`、`misses=5`、`stores=5` |
| Web release dependency-aggregate second | same source / same `CompilerSession` | `compile_ms=4`、`prewarm_ms=0`、`wasm_call_ms=4`、`prewarm_surface_hits=1` |
| Web release dependency-aggregate body-only edit | return literalだけを変更 + same `CompilerSession` | `compile_ms=4`、`prewarm_ms=0`、`wasm_call_ms=4`、`prewarm_surface_hits=2` |

2026-05-28 の修正で、dependency aggregate query には同一 traversal 内の memo を追加した。diamond dependency graph で同じ configured stdlib module を何度も展開しないためである。ただし、RPN のような stdlib-heavy source では、この query を `CompilerSession.prewarm_loader_cache_for_source` から同期実行すると private implementation import closure を広く歩き、compile phase が 120 秒 timeout になることを確認した。

したがって、この checkpoint の成果は「将来の semantic cache key」として保持し、Web playground の compile 前 prewarm からは外した。aggregate first の compile time を直接下げる効果はまだない。次段階の typed public surface / Resource IR summary cache が実際にこの key を消費する段階で、public surface 境界と invalidation 範囲を再確認して接続する。

## Web playground release / compiled-output cache checkpoint

2026-05-28 の checkpoint では、Web playground で compile が終了しない体感問題を先に緩和するため、build artifact と同一 session 再compileの境界を修正した。

追加した境界:

- `trunk build` の通常実行が Rust/WASM release artifact を作るように、`Trunk.toml` の `[build].release = true` を固定した。HTML の Rust asset も `data-cargo-profile="release"` を持つ。
- NEPL source profile の既定値は `BuildProfile::Debug` に固定した。明示的な `--profile release` / `compile_source_with_*_profile(..., "release")` だけが `#if[profile=release]` を有効化する。
- `CompilerSession.compile_outputs_with_vfs` と `CompilerSession.compile_source_with_vfs_and_profile` は、entry path、source、compile VFS、NEPL source profile、WAT comment mode を key にして `CompiledWasm` を保持する。
- cache value は wasm bytes と NEPL WAT debug comment だけであり、`SourceMap`、typed HIR、`TypeId`、Resource IR summary、diagnostic span は保持しない。
- VFS key は `merge_vfs_sources` と同じ正規化規則で作る。path が空の entry は除外し、非文字列 content は compile 側と同じく空文字列として扱う。
- cache limit は 8 entries に抑え、Web session が無制限に wasm bytes を保持しないようにする。
- `clear_loader_cache()` は loader cache と同時に compiled-output cache と hit/store counter を消す。
- `loader_cache_stats_json()` は `compiled_output_cache_hits` / `compiled_output_cache_stores` を返す。

追加測定:

| case | command / artifact | result |
|---|---|---|
| Web release RPN prewarm direct | `CompilerSession.prewarm_loader_cache_for_source("/virtual/entry.nepl", rpn)` | `prewarm_ms=267`、`prewarm_count=16`、dependency aggregate counters `0` |
| Web release RPN doctest first | `web/examples/rpn.nepl` + same `CompilerSession` | `compile_ms=8976`、`prewarm_ms=193`、`wasm_call_ms=8783`、`compiled_output_cache_hits=0`、`stores=2` |
| Web release RPN doctest second | same source / same `CompilerSession` | `compile_ms=1`、`prewarm_ms=1`、`wasm_call_ms=0`、`compiled_output_cache_hits=1`、`stores=2` |

この checkpoint は「同じ入力の再compile」を 10ms 未満にするが、初回 compile はまだ 9 秒級である。RPN では `resource_initialized_raw_init_summaries` と `resource_initialized_function_checks` が支配的であり、i32 scalar query cache 後も full Resource IR pipeline が残る。したがって次段階は、dependency aggregate public surface hash と typed public signature hash を使う Resource summary namespace の下で、変更されていない stdlib / user function の summary value を再利用する。

現在の compiled-output cache key は stale hit を避けるため、entry source と compile VFS 全体を含める。これは安全だが、未使用の editable `.nepl` file が変わっただけでも false miss になる。依存 closure に基づく output cache key は、typed public surface / import graph cache と同じ invalidation 証明が必要なので、この checkpoint では実装しない。

2026-05-28 の semantic source key checkpoint では、compiled-output cache key の source 部分を raw text から `nepl-core::source_cache_key::compiled_source_cache_key_part` へ差し替えた。この関数は lexer token stream を使う pure function であり、ordinary comment、doc comment、position-only span change は compiled output の key に入れない。一方で `Indent` / `Dedent` / `Newline`、directive、raw wasm / llvm text、`mlstr` line、literal / identifier payload は保持する。lexer diagnostic がある source では raw text key へ戻し、以前の successful compile が新しい lexer error を隠さないようにする。

doc comment を無視するのは compiled output cache だけの契約である。documentation extraction、doccomment test、nm document 生成の cache key としてこの key を流用してはならない。これはコメント増加を性能上の不利益にしないための境界であり、コメントの意味を使う機能では別の documentation-oriented key を使う。

追加測定:

| case | command / artifact | result |
|---|---|---|
| Web release RPN base | `web/examples/rpn.nepl` + same `CompilerSession` | `compile_ms=9013`、`prewarm_ms=235`、`wasm_call_ms=8778`、`compiled_output_cache_hits=0`、`stores=2` |
| Web release RPN ordinary comment edit | same source with ordinary comment-only line + same `CompilerSession` | `compile_ms=2`、`prewarm_ms=1`、`wasm_call_ms=0`、`compiled_output_cache_hits=1` |
| Web release RPN doc comment edit | same source with doccomment text edit + same `CompilerSession` | `compile_ms=1`、`prewarm_ms=0`、`wasm_call_ms=1`、`compiled_output_cache_hits=2` |
| Web release RPN code edit | same source with string literal edit + same `CompilerSession` | `compile_ms=8347`、`prewarm_ms=0`、`wasm_call_ms=8347`、`compiled_output_cache_hits=2`、`stores=3` |

この checkpoint で、comment-only / doccomment-only の微小変更は 10ms 未満になった。code edit はまだ full compile になり、raw initialization summary と function check が支配的であるため、次段階の Resource summary value reuse は引き続き必要である。

## Resource IR scalar query cache checkpoint

2026-05-28 の Resource IR 側 checkpoint では、i32 scalar condition / alias / offset query を `I32ConditionQueryContext` に集約し、summary 内で同じ純粋 query を繰り返さないようにした。

追加した境界:

- direct i32 value、bounded offset value、scalar alias、offset source / target / reachable の query を context-aware helper へ分けた。
- return fact collection は同じ context を共有し、条件判定と return fact 変換で同一の alias / offset graph を再探索しない。
- raw initialization summary は、signature relevance、raw alias summary、direct raw op、関連 callee の dependency closure で対象を絞る。ただし reference parameter から seed される raw initialization 前提を落とさないよう、signature relevance を先に残す。

追加測定:

| case | command / artifact | result |
|---|---|---|
| native release RPN static check | `target/release/nepl-cli.exe --check -i examples/rpn.nepl --target std --stdlib-root stdlib` | `resource_static_check=9202ms`、`resource_initialized_i32_scalar_summaries=2012ms`、`resource_initialized_raw_init_summaries=2520ms`、`resource_initialized_function_checks=3730ms` |

subagent review 後に、Branch / Match の sibling variant return facts、collection slot operation の raw initialization relevance、offset/constant derived equality を戻した。これにより correctness-first の path preservation が増え、i32 scalar summary は最小値の 571ms から 2012ms へ戻ったが、false negative を避けるための必要な修正である。total static check はまだ 9 秒台であり、raw init / function check の path-sensitive exploration が次の主要ボトルネックである。

2026-05-28 の追加 checkpoint では、Resource IR 内の純粋 query と局所証明をさらに狭めた。`CollectionSlotTransformRange` を含まない関数では local transform-range certificate の候補を構築しない。これは同じ関数内の transform-range op だけが消費する局所証明なので、消費先がない関数では静的検査の結果を変えない。また、i32 scalar return leaf relation 収集では、leaf pair ごとに `I32ConditionQueryContext` を作り直さず、relation 収集全体で共有する。alias / offset 到達性は同じ raw alias graph に対する純粋 query であり、context 共有により同じ探索を繰り返さない。

追加測定:

| case | command / artifact | result |
|---|---|---|
| native release RPN static check after Resource IR query pruning | `target/release/nepl-cli.exe --check -i examples/rpn.nepl --target std --stdlib-root stdlib` | `resource_static_check=8389ms`、`resource_initialized_i32_scalar_summaries=1372ms`、`resource_initialized_raw_init_summaries=2613ms`、`resource_initialized_function_checks=3470ms` |

`I32ConditionQueryContext` 全体を `BTreeMap` memo に置き換える案と、loop initialized range の body guard を先行する案は実測で悪化したため採用しない。今回の checkpoint で i32 scalar summary は軽くなったが、初回 compile はまだ 0.5 秒未満から遠い。次段階は typed public signature table を semantic invalidation 境界にし、Resource IR summary を function hash と dependency typed public signature hash で再利用する。

2026-05-28 の追加 checkpoint では、Resource IR initialized check の path-sensitive replay をさらに限定した。Branch / Match / call return 後の `path_alternatives` は診断精度を保つための精密化だが、operation ごとに全 path へ同じ処理を replay すると scalar-heavy 関数で探索が膨らむ。そこで、merged state だけで処理してもよい operation を「入力 place を読まず、診断を生成せず、fresh temporary にだけ確定 i32 scalar fact を置く式」に限定して追加した。

許可した式:

- `LiteralI32`
- `LayoutSizeOf`

許可条件:

- `output.root` が `Temporary` である。
- `output.projections` が空である。

local や projection 付き output は、path ごとに alias / scalar fact が異なり得るため対象外にした。一般の `Literal`、`DeclareLocal`、`FunctionValue` も、str storage layout や function alias precision、fresh local 前提への依存があるため今回の slice には含めない。

追加測定:

| case | command / artifact | result |
|---|---|---|
| native release RPN static check after merged literal fast path | `target/release/nepl-cli.exe --check -i examples/rpn.nepl --target std --stdlib-root stdlib` | `resource_static_check=8033ms`、`resource_initialized_i32_scalar_summaries=1309ms`、`resource_initialized_raw_init_summaries=2509ms`、`resource_initialized_function_checks=3317ms` |

この変更は Resource IR summary cache ではなく、同一 function check 内の path replay 削減である。初回 compile の 0.5 秒未満にはまだ届かないが、`resource_initialized_function_checks` が 3470ms から 3317ms に下がったため、path-sensitive exploration が実際に残り hot path であることを確認した。次段階は、subagent review で安全境界を確認した typed public signature table を arena 非依存の semantic cache key として構築し、typed HIR / `TypeId` / Resource IR を直接保持せずに stdlib summary 再利用へ接続する。

2026-05-28 の typed public signature checkpoint では、typecheck 成功時に `TypedPublicSignatureTable` を生成するようにした。これは後続 cache をまだ再利用しない観測用 artifact であり、`TypeId`、`Span`、`SourceMap`、typed HIR、Resource IR を保持しない。table value は stable text entry と deterministic hash だけで構成する。

table に含める内容:

- public callable の名前、関数型 signature、`noshadow`。
- public struct の type parameter、field type、constructor policy。
- public enum の type parameter、variant payload type。
- public trait の type parameter、capability、method signature。
- impl header の trait application と target type。

table には関数本体を含めない。これにより body-only edit は semantic cache key を変えず、public callable type edit は key を変える。現在は `TypeCheckResult.public_signatures` として露出し、regression では body-only edit で `stable_hash` が不変、public callable return type edit で変化することを固定している。

この checkpoint は Resource IR summary reuse にはまだ接続しない。次段階では、loader の dependency aggregate public surface hash と typed public signature hash を組み合わせる namespace key を compiler pipeline へ接続し、stdlib module の typed check / Resource IR summary cache の invalidation key として使う。

2026-05-28 の追加 staging では、`TypedPublicSignatureTable` を `TypeCheckResult` から `PreparedProgram` まで通すようにした。これは cache value の再利用ではなく、Resource IR summary cache key を構築するための入力を compiler pipeline の後段へ運ぶだけである。`PreparedProgram` に保持される table も stable text / hash のみで、typed HIR や `TypeId` を session cache value として保存するものではない。

2026-05-28 の Resource summary namespace key checkpoint では、`ResourceSummaryCacheNamespaceKey` を `PreparedProgram` に追加した。この key は `neplg2-resource-summary-cache-namespace-v1`、target、profile、typed public signature hash、任意の dependency public surface hash を決定的に hash 化する。最初の checkpoint では compile path へ dependency aggregate をまだ渡さず `None` とし、Web / Node prewarm hot path でも dependency aggregate を同期計算しない境界だけを固定した。

この key は Resource IR summary cache の「名前空間」を分けるための staging artifact であり、Resource IR summary value の hit / store はまだ実装しない。`TypeId`、`Span`、`SourceMap`、typed HIR、Resource IR body、diagnostic span、codegen fragment は key に保存しない。次段階では、この namespace key に function body hash、source capability policy hash、generic type-argument hash を組み合わせ、到達 function ごとの summary reuse へ進める。

regression では、public function の body-only edit で namespace key が変わらず、public callable return type edit で変わることを固定した。さらに dependency public surface hash option が変われば同じ typed public signature hash でも別 namespace になることを固定し、loader から dependency aggregate を接続する次段階の入力境界を明確にした。

native release RPN stage-only 測定は、`resource_typecheck=121ms`、`resource_initialized_i32_scalar_summaries=1270ms`、`resource_initialized_raw_init_summaries=2187ms`、`resource_initialized_function_checks=3063ms`、`resource_static_check=7353ms` だった。この checkpoint は namespace key の staging であり、summary value reuse による初回 compile 0.5 秒未満はまだ達成していない。

2026-05-28 の dependency namespace connection checkpoint では、session-backed bundled stdlib compile path だけが loader の `root_dependency_aggregate_public_surface_hash_for_source_with_cache` を呼び、その結果を `ResourceSummaryCacheNamespaceKey` へ渡すようにした。汎用 `CompileOptions` は増やさず、明示 helper `compile_module_with_source_map_artifact_options_and_dependency_public_surface_hash` で Web session path からだけ入力する。

この接続は prewarm では実行しない。`nodesrc/test_run_test_compiler_session.js` では、`prewarm_loader_cache_for_source` の前後で dependency aggregate counter が増えず、compile call の前後でだけ増えることを固定した。compiled-output cache hit を返す経路でも、dependency aggregate counter が増えないことを stub で固定した。`/stdlib` overlay や forced stdlib VFS では loader cache が bypass されるため、namespace key には dependency public surface hash を渡さない。

Web release RPN doctest 測定では、初回 `compile_ms=9095`、`prewarm_ms=210`、`wasm_call_ms=8884`、2 回目 `compile_ms=1`、`wasm_call_ms=0` だった。prewarm cache stats では dependency aggregate hit/miss/store が増えず、compile 本体で `dependency_aggregate_public_surface_hash_hits` が `0 -> 4`、`misses` が `4 -> 122`、`stores` が `4 -> 122` になった。これは Resource summary value reuse ではないため、初回 compile 0.5 秒未満はまだ未達である。

2026-05-28 の variant-param summary checkpoint では、raw initialization summary 内の variant-param collector を呼ぶ前に、return value を直接 output とする top-level `Branch` が block に存在するかを確認するようにした。collector は現時点でその `Branch` だけを facts 抽出対象にしているため、該当しない block で `ResourceCheckEngine` prefix replay を起動しても新しい variant-param facts は得られない。このため、観測できる証明境界を広げずに探索空間だけを削減する。

native release RPN stage-only 測定では、直前 checkpoint の `resource_initialized_raw_init_summaries=2421ms`、`resource_initialized_function_checks=3224ms`、`resource_static_check=7776ms` に対し、今回 checkpoint は `resource_initialized_raw_init_summaries=2281ms`、`resource_initialized_function_checks=3090ms`、`resource_static_check=7443ms` だった。初回 compile 0.5 秒未満にはまだ届かないため、次は typed public signature hash と dependency public surface hash を使う Resource IR summary cache に進む。

2026-05-28 の duplicate path dedup / string byte predicate checkpoint では、Resource IR initialized check の `ResourcePathAlternatives::from_states` で budget 超過時に完全重複候補だけを先に落とすようにした。subagent review では、全候補が同一でも既存 merge が collection slot range の弱化や raw alias 正規化を行うことを確認したため、merge lattice を clone で省略する fast path は採用しない。dedup は exact duplicate だけを落とし、残った候補は従来どおり conservative merge に通す。

同 checkpoint では、`string/byte_index` に private `string_byte_or_invalid` と低水準 `string_byte_is_ascii_space` を置き、slice から使う public predicate は `string/search/compare` の `str_byte_is_ascii_space_at` にした。sentinel 値と raw read 境界は byte_index module 内部に閉じ、search predicate はその証明済み predicate を高水準 module へ渡す facade になる。`str_trim` は loop 本体で `Option` branch を直接展開せず、この checked search predicate だけを使う。これにより raw read の範囲証明は byte_index module に閉じつつ、slice module が検索 predicate を使う責務境界を維持する。

native release RPN stage-only 測定では、safe revision 後に `resource_initialized_i32_scalar_summaries=1256ms`、`resource_initialized_raw_init_summaries=2549ms`、`resource_initialized_function_checks=3139ms`、`resource_static_check=7870ms` だった。per-function timing run では `str_trim` の function check が before の `1018ms` から `699ms` へ下がった。一方で全体はまだ 7 秒台で、raw init summary と function check が支配的である。

この checkpoint も Resource summary value reuse ではない。subagent review では、既存 summary struct が `TypeId` や `Span` を含むため、そのまま長寿命 cache value にしないことを確認した。次段階では namespace key に function body hash、generic type-argument hash、source capability policy hash、summary kind/version を追加し、arena 非依存の stable mirror value へ落としてから stdlib summary reuse に接続する。

2026-05-28 の追加 review では、RPN の call graph pruning が保守的に全関数へ倒れているのではなく、monomorphized function 290 件のうち 287 件が実際に entry から到達していることを確認した。したがって、次の大きな削減余地は import pruning ではなく、到達済み関数の Resource summary value を namespace 内で再利用することである。

Resource summary value cache の owner は `LoaderSessionCache` ではなく `CompilerSession` の別 field とする。`LoaderSessionCache` は source / AST / loader surface の未型付け artifact を持つ層であり、Resource IR summary は typed public signature、dependency public surface、target/profile、source capability policy、generic type argument、summary kind/version に依存する。loader cache に入れると、未型付け artifact と Resource IR proof artifact の invalidation 境界が混ざる。

初期 cache value は既存 summary struct 全体ではなく、arena 非依存の stable mirror として定義する。最初の対象は `CollectionSlotLifecycleSummaryOp::DropTraversal` と `ForallInitializedRange` に限定する。これらは drop traversal と initialized range proof の小さい事実へ落としやすく、`RawCellInitializationFunctionSummary` 全体のように session-local `PlaceProjection` / `TypeId` / raw alias state を広く含まない。

初期実装で store する条件:

- namespace key があり、summary kind/version が一致する。namespace key は target/profile、typed public signature hash、dependency public surface hash の外枠であり、単独で summary value cache の key にはしない。
- per-summary-value key は structured key として保持し、namespace key、canonical function identity、function body hash、function-local type parameter boundary、generic type-argument hash、source capability policy hash、summary kind/version を分けて持つ。短い hash だけを保存せず、debug / regression で stale hit の原因を追える形にする。
- value は stable summary place / projection / type key、known i32、expected type、element stride、`StateOnly` / `LoadedValueDrop` proof のような arena 非依存データだけで表現できる。
- nominal type は module/path/definition identity まで含む qualified stable type key にできる場合だけ store する。qualified identity が得られない場合は、型名文字列が一致しても別定義へ stale hit し得るため bypass に倒す。
- label 付き generic variable は label だけでは store key として不十分である。function-local type parameter ordinal / boundary と generic type-argument hash を per-value key に含め、hit 後に現在 compile の type parameter へ再投影できる場合だけ store する。
- cache hit 後も現在 compile の Resource IR place / type context へ再投影できる。

初期実装で store しない条件:

- `CertifiedSlots`、`TransformRange`、`Event`、`Relocate`、return path facts、Merge / Loop にまたがる proof。
- `RawCellInitializationFunctionSummary` 全体、raw alias graph、session-local `TypeId`、`Span`、`SourceMap`、typed HIR、diagnostic span を含む value。
- `expected_ty` や `LoadedValueDrop` proof 内の型が stable type key へ正規化されておらず、現在 compile の `TypeCtx` へ再投影できない value。
- stale な callee-local place を保持したまま現在 compile の Resource IR context へ再投影しなければならない projection。`SummaryOffset::Unknown` / `ResourceOffset::Unknown` は exact offset を要求しない conservative proof value として再投影できるため、この条件には含めない。
- nominal type identity が unqualified name だけで、module/path/definition identity と結びついていない value。
- label 付き generic variable が function-local type parameter boundary と type-argument hash に結びついていない value。
- source capability policy hash、generic type-argument hash、target/profile、dependency public surface hash のいずれかが未確定の compile。
- `/stdlib` overlay、forced stdlib VFS、local stdlib override のように bundled stdlib proof と source が一致しない compile。

`CompilerSession.loader_cache_stats_json()` とは別に、Resource summary value cache は `resource_summary_value_hits` / `misses` / `stores` / `bypasses` と、summary kind 別の hit/store/bypass counter を持つ。ここでの `hit` は「同じ stable value が既に存在し、現在 compile の type/place boundary へ逆投影できる候補 hit」であり、fixed-point worklist を skip して compile work を削減したことまでは意味しない。実際に summary op を replay して仕事量を減らす段階では、`resource_summary_value_replay_hits` / `resource_summary_value_replayed_ops` / `resource_summary_value_recomputed_ops` を別 counter として増やす。

2026-05-28 の implementation staging では、`ResourceSummaryValueCache` と `ResourceSummaryValueCacheStats` を `nepl-core::resource` に追加し、`CompilerSession` が `LoaderSessionCache` とは別 field として所有する形にした。`loader_cache_stats_json()` は `resource_summary_value_*`、`resource_summary_value_replay_*`、`resource_summary_value_drop_traversal_forall_*` の counter を返す。store/hit MVP では逆投影可能な candidate を観測し、complete leaf entry preseed checkpoint から dependency-free な entry だけを fixed-point worklist 前に replay する。

2026-05-28 の bypass instrumentation checkpoint では、`CompilerSession` の compiled-output cache miss から実際に走る compile path だけへ `ResourceSummaryValueCache` を渡すようにした。`check_resource_initialized_moves` の既存 API は残し、session-backed compile だけが `check_resource_initialized_moves_with_summary_cache` を使う。これにより CLI や stateless Web API の挙動を変えず、session cache の観測を Web / Node の timing JSON に限定して追加できる。

この checkpoint で記録するのは、worklist 固定点が収束した後の最終 `CollectionSlotLifecycleFunctionSummary` に top-level op として残る `CollectionSlotLifecycleSummaryOp::DropTraversal` かつ `ForallInitializedRange` の候補だけである。return path facts や `Merge` / `Loop` 内の leaf は、初期 MVP の store 対象外なのでこの counter へ含めない。これは「安全に保存できる候補が実 workload にどれだけ存在するか」を compiled-output cache とは別に測るための段階であり、`TypeId` / `Span` / `SourceMap` / typed HIR / Resource IR body を長寿命 value に保存しない方針を維持する。

2026-05-28 の stable mirror conversion checkpoint では、`DropTraversal + ForallInitializedRange` を既存 summary struct のまま cache value にせず、`ResourceSummaryStableDropTraversalForallValue` へ変換する足場を追加した。型は `TypeId` ではなく `ResourceSummaryStableTypeKey` へ変換し、無名 type variable や cycle のように arena slot へ依存する型は保存候補から外す。`SummaryPlace` / projection / symbolic offset / known i32 / `StateOnly` / `LoadedValueDrop` proof も stable mirror 型へコピーする。現 checkpoint ではまだ `BTreeMap` への store/hit は行わず、bypass counter も stable mirror へ変換できた top-level 候補だけを数える。

2026-05-28 の stable mirror split checkpoint では、stable mirror 変換を `ResourceSummaryValueCache` の sibling module ではなく、`resource_summary_value_cache::stable_mirror` private submodule へ分離した。これは cache owner の public/internal API を増やさず、store/hit 実装が入る前に「統計と map 所有」と「session-local value から stable value への変換」を別責務に分けるためである。当初は `SummaryOffset::Unknown` を exact offset へ再投影できない projection として拒否していたが、2026-05-31 の raw-init stable entry checkpoint で、`Unknown` は stale な local identity を持たない conservative proof value として保存・再投影できる境界へ変更した。nominal type key と generic variable key は、per-value key に qualified definition identity / function type parameter boundary / generic type-argument hash を含めるまで store 対象にしない。

2026-05-28 の structured key staging checkpoint では、`resource_summary_value_cache::key` private module に per-summary-value key の型だけを追加した。これは map store/hit ではなく、namespace hash、function identity、function body hash、function-local type parameter boundary hash、generic type-argument hash、source capability policy hash、summary kind/version を裸の hash blob にまとめず、field として分けたまま扱うための足場である。追加 review では、function body hash と source capability policy hash が compiler pipeline から渡るまで store/hit を入れないべきだと確認したため、この checkpoint では key 型と invalidation input regression だけを固定する。

2026-05-28 の source capability policy hash checkpoint では、`SourceMap::source_capability_policy_hash_for_file(file_id)` を追加した。これは source capability を広く許可する query ではなく、Resource summary value key に入れるための deterministic fingerprint である。use-site proof は byte range を持つため、canonical path と source content hash を必ず hash に含め、同じ proof range が別 source に流用されないようにする。source hash は caller に渡させず `SourceMap` 内の source text から計算するため、別 source の hash や sentinel 値を key に混ぜる経路を作らない。現時点では function body hash と Resource summary store/hit にはまだ接続せず、path/source hash/use-site/span/order の regression だけを固定する。

2026-05-28 の function body hash staging checkpoint では、`resource_summary_value_cache::stable_type_key` を `stable_mirror` から独立させ、summary value と function body hash が同じ TypeId 安定化規則を共有できるようにした。hash writer も `stable_hash` に分け、per-summary-value key と body hash が同じ区切り付き FNV-1a 系 writer を使う。`resource_summary_value_cache::body_hash` は `ResourceFunction` の `Span` を無視し、`TypeId` を stable type key へ変換し、temporary / block id は body 内 ordinal に正規化する。`StorageId` は owner/checker state 側の割当に由来するため、数値を直接 key に入れず、body traversal で最初に現れた順の ordinal へ正規化する。raw body は本文が `ResourceFunction` に残らないため、raw body/source hash を key へ追加するまでは拒否する。nominal type は後続 checkpoint で source path / definition fingerprint 付き identity を導入し、identity がある場合だけ stable type key へ変換する。body hash namespace はこの型 key 境界変更に合わせて v2 とした。

同日の bypass candidate connection checkpoint では、最終 `CollectionSlotLifecycleFunctionSummary` の top-level `DropTraversal + ForallInitializedRange` を候補として数える前に、対応する `ResourceFunction` の body hash が作れることも確認するようにした。これにより、stable mirror value だけは作れても raw body / storage root / nominal type などで per-summary-value key を安全に作れない関数は、store/hit 実装前の観測 counter からも外れる。

type boundary hash checkpoint では、per-summary-value key の `type_parameter_boundary_hash` と `generic_type_argument_hash` を作る private staging module を追加した。type parameter boundary は `summary.type_params` を基準にし、unbound かつ label 付きの type variable だけを許可する。hash には arity、ordinal、label、copy/clone/drop capability を含め、同じ stable parameter key が重複した場合は再投影先が曖昧になるため拒否する。anonymous variable、bound variable、concrete type、identity なし nominal type は boundary として保存しない。generic type argument hash は順序付きの argument list として扱い、各 argument を `ResourceSummaryStableTypeKey` に変換できる場合だけ作る。source path / definition fingerprint 付き identity を持つ nominal argument は保存対象にできる。generic argument hash namespace はこの受け入れ範囲変更に合わせて v2 とした。

function identity gate checkpoint では、top-level `DropTraversal + ForallInitializedRange` の候補を数える前に `ResourceSummaryFunctionIdentity::from_resource_function` も通すようにした。canonical symbol と origin name が空の function は、compile session 間で対応する callable 境界を特定できないため、body hash や type boundary hash が作れても store 候補として観測しない。

candidate key builder checkpoint では、`resource_summary_value_cache::candidate_key` private staging module を追加し、per-value key を作るための全入力を一箇所で合成する境界を固定した。builder は namespace hash、source capability policy hash、function identity、function body hash、function-local type parameter boundary hash、generic type argument key、stable mirror value がすべて作れる場合だけ `ResourceSummaryValueCacheKey` を返す。namespace hash と source capability policy hash は型名付き wrapper で受け、`0` や empty を未計算 sentinel として扱わない。generic type argument は `NonGeneric` / `TemplateBoundaryOnly` / `KnownInstantiation` の明示 enum にし、現行 summary が concrete call-site args を持たないことと、将来の instantiated cache key が実引数を取り忘れないことを両立させる。

source policy context checkpoint では、`ResourceSummaryValueCacheContext` を追加し、compiler pipeline で作った `ResourceSummaryCacheNamespaceKey::stable_hash` と `SourceMap::source_capability_policy_hash_for_file` の結果だけを Resource initialized check へ渡すようにした。Resource checker には raw `SourceMap` を渡さず、`FileId -> source policy hash` の小さな context だけを渡す。context は `ResourceFunction` / block / op / terminator / nested control-flow op / match arm の distinct file id を集め、対応する source policy hash がすべて存在する場合だけ per-function source policy hash を作る。`Span::dummy()` は source file 0 とみなさず無視し、実 source policy が取れない候補は no-store / bypass に倒す。現 checkpoint でも map store/hit はまだ行わず、keyable candidate だけを bypass counter として観測する。

2026-05-28 の store/hit MVP checkpoint では、`ResourceSummaryValueCache` が `DropTraversal + ForallInitializedRange` の stable mirror value を `CompilerSession` 内の map に保存するようにした。これは Zenn 方針の「試作段階でも根本の構造を直す」範囲の変更であり、timeout 延長、静的検査の削除、Resource proof coverage の削減ではない。cache value は純粋な Resource summary query の結果だけであり、`LoaderSessionCache` とは別 field として保持する。

reverse projection checkpoint では、hit 候補にする前に `ResourceSummaryStableDropTraversalForallValue` を現在 compile の `CollectionSlotLifecycleSummaryOp::DropTraversal` へ戻せることを確認する。`ResourceSummaryTypeReprojection` は builtin 型、function-local type parameter boundary、現在の function signature から stable type key と現在の `TypeId` の対応を作り、曖昧な generic boundary、範囲外 parameter index、projection layout mismatch、stride mismatch、identity 不明または現在 compile で一意に対応しない nominal type は miss/bypass に倒す。これは candidate hit の安全境界であり、fixed-point worklist の seed / skip はまだ行わない。

complete leaf entry checkpoint では、cache value を個別 leaf の dedup 可能な `Vec` ではなく、順序と重複を保持する `ResourceSummaryStableDropTraversalForallLeafEntry` へ変更した。fixed-point skip で使える entry は function summary 全体の surface を復元できる必要があるため、`ops` がすべて top-level `DropTraversal + ForallInitializedRange` で、`return_transfers` / `return_slots` / `return_ranges` / `return_paths` が空の場合だけ store/hit 候補にする。依存関数を持つ caller、`IndirectCall` を含む関数、`CertifiedSlots` / `Merge` / `Loop` / `TransformRange` などを含む summary は、dependency fingerprint と追加 stable mirror を設計するまで bypass に倒す。

complete leaf entry preseed checkpoint では、dependency-free かつ `IndirectCall` を持たない関数だけを対象に、保存済み entry を現在 compile の `CollectionSlotLifecycleSummaryOp` 列へ逆投影してから `summaries` に先に入れ、`SummaryWorklist` の初期 pending から外す。これにより replay できた関数は通常の fixed-point recompute を行わず、`resource_summary_value_replay_hits` と `resource_summary_value_replayed_ops` が op 数ぶん増える。通常 recompute した complete leaf entry は `resource_summary_value_recomputed_ops` に数え、candidate hit だけを示す `resource_summary_value_hits` と実 compile work skip を示す replay counter を分離する。

raw-init param facts checkpoint では、`RawCellInitializationParamCell` と simple projection の `RawCellReleaseParamRequirement` を complete leaf entry として stable mirror 化する足場を追加した。store 対象は `IndirectCall` なし、return / byte-range / variant facts なしの summary に限定する。依存関数を持つ caller は、dependency closure hash を raw-init key に含められる場合だけ保存する。raw alias graph、partial summary、diagnostic span、`TypeId`、`Span`、`SourceMap` は保存しない。

同 checkpoint の追加 review では、raw-init preseed が実際に fixed-point worklist を skip する経路になったため、Rust 側 regression を追加した。保存済み entry が同じ summary surface として再投影されること、function body hash、source policy hash、signature type boundary のいずれかが変わると preseed miss になり通常 worklist に戻ることを固定する。raw body は source policy hash があっても本文が `ResourceFunction` に残らないため、body hash では引き続き拒否する。

qualified nominal type identity checkpoint では、`TypeCtx` に `NominalStableTypeIdentity` を追加した。identity は `TypeId` / `Span` ではなく、`SourceMap` から得た source path、定義 kind、定義名、arity、field / variant / type parameter から作る definition fingerprint を持つ。`typecheck::driver` は `StructDef` / `EnumDef` の登録時にこの identity を `TypeCtx` へ保存し、checkpoint rollback では identity map も戻す。

`ResourceSummaryStableTypeKey` は、`Struct` / `Enum` / resolved `Named` がこの identity を持つ場合だけ nominal key を作る。未解決 `Named` placeholder、identity のない nominal definition、cycle を含む definition surface は従来どおり bypass に倒す。backend scalar の `u32` / `i64` / `u64` / `f64` は既存 `TypeKind::Named` を使う compiler-owned scalar なので、nominal identity とは別の builtin scalar key として扱う。

この変更で、generic type argument hash と function body hash は identity 付き nominal type を保存候補にできる。subagent review で指摘された name-only stale hit を避けるため、単なる `Struct.name` / `Enum.name` は cache key に使わない。source path と definition fingerprint が得られない場合は、性能より安全を優先して no-store / miss に戻る。

qualified nominal type identity checkpoint 前の RPN 実測では、raw-init param facts は `raw_init_param_facts_bypasses=225`、`raw_init_param_facts_stores=0` だった。subagent review では、store 対象が狭いことに加えて `ResourceSummaryStableTypeKey` が `Named` / `Struct` / `Enum` を拒否しているため、nominal 型を多く含む stdlib summary が candidate 化できない可能性が高いと確認した。このため、qualified nominal type identity を別 issue `ISS-20260528T110220373Z-RESOURCE-SUMMARY-CACHE-NEEDS-QUALIFI-08D1AA04` に分離し、上記 checkpoint で `TypeCtx` identity と stable type key 受け入れ境界を実装した。

dependency closure raw-init checkpoint では、raw-init summary が user call の callee summary を取り込むことを前提に、direct dependency だけではなく reachable dependency closure の function identity、body hash、source capability policy hash、function-local type boundary hash を raw-init key へ含めるようにした。これにより、依存先 implementation edit 後に caller の cached raw-init facts が stale hit する経路を避けながら、dependency-bearing complete leaf summary も保存対象にできる。

同 checkpoint では、raw-init cache の bypass counter を reason 別に分けた。RPN same-session code-edit の release Web 測定では、初回 `raw_init_param_facts_stores=2`、2 回目 `raw_init_param_facts_hits=2` / `resource_summary_value_replay_hits=2` まで到達した。これは `ISS-20260528T110220373Z-RESOURCE-SUMMARY-CACHE-NEEDS-QUALIFI-08D1AA04` の完了条件である store / hit 非 0 を満たす。

一方で、同測定では `raw_init_param_facts_unstable_key_bypasses=176`、`raw_init_param_facts_incomplete_leaf_bypasses=37`、`raw_init_param_facts_reprojection_bypasses=10` が残った。これは nominal type identity の不足ではなく、dependency closure 内の key 化不能関数、byte-range / variant / return facts を含む complete raw-init mirror 不足、generic nominal instantiation の再投影不足であるため、それぞれ別 issue に分離した。

raw body dependency key checkpoint では、このうち `raw_init_param_facts_unstable_key_bypasses=176` を `0` まで減らした。RPN の dependency closure には `core/mem/raw` 系の raw body callee が含まれるが、raw body 本文は `ResourceFunction` に直接残らない。そこで `resource_function_body_hash` は `ResourceTerminator::RawBody` の backend kind を hash し、source body text と raw memory capability use-site は `source_capability_policy_hash_for_function` 経由で key に含める契約へ整理した。source capability policy hash は source path、source content hash、use-site set を含むため、raw body text や boundary の変更では dependency closure hash が変わる。

同 checkpoint では、dependency closure hash failure の counter を dependency graph / identity / body hash / source policy / type boundary へ分割した。RPN same-session code-edit の release Web 測定では、初回 `raw_init_param_facts_stores=2`、2 回目 `raw_init_param_facts_hits=2` / `resource_summary_value_replay_hits=2` を維持しながら、`raw_init_param_facts_unstable_key_bypasses=0` になった。残件は `raw_init_param_facts_unstable_entry_bypasses=119` と `raw_init_param_facts_reprojection_bypasses=67` であり、前者は `ISS-20260528T125932150Z-RESOURCE-SUMMARY-RAW-INIT-STABLE-ENT-AE09D7D6`、後者は `ISS-20260528T123956303Z-RESOURCE-SUMMARY-TYPE-REPROJECTION-N-78929A8E` で扱う。

raw-init stable entry checkpoint では、`RawCellReleaseParamRequirement` の suffix に含まれる `StorageOffset` を stable mirror 化した。parameter-relative な `ResourceOffset::{Symbolic, ScaledSymbolic, Offset, ScaledOffset}` は stable place として保存し、callee-local operand で現在 compile へ再投影できないものは stale local place を保存せず `Unknown` へ正規化する。既存の raw address overlap 判定では symbolic / offset / unknown はいずれも may-overlap として扱うため、この正規化は検査削除ではなく、保存できない identity を conservative proof value へ落とす変更である。

同 checkpoint の release Web RPN same-session code edit 測定では、初回 `compile_ms=9794`、2 回目 `compile_ms=8668` だった。raw-init param facts は初回 `stores=23`、2 回目 `hits=23` / `resource_summary_value_replay_hits=23` となり、`raw_init_param_facts_unstable_entry_bypasses` は `119 -> 0` になった。残件は `raw_init_param_facts_reprojection_bypasses=165` と `raw_init_param_facts_incomplete_leaf_bypasses=37` であり、次は instantiated generic nominal mapping と byte-range / variant / return facts の complete mirror を進める。

同 review で、`ResourceFunction.name` には重複定義用の source span mangle が入る場合があり、cache key の false miss を増やすと確認した。`ResourceSummaryFunctionIdentity` は `__def<file>_<start>_<end>` の定義 span component を key から外し、function body hash と source capability policy hash で実体差分を追う。これは stale hit を増やすためではなく、SourceMap / Span に依存しない function identity へ近づけるための正規化である。

type reprojection checkpoint では、`ResourceSummaryTypeReprojection` が generic nominal definition tree を現在 compile の instantiated boundary へ戻す規則を拡張した。function-local type parameter boundary は strict に登録し、同じ stable generic key が別 `TypeId` へ割れる場合は引き続き拒否する。一方で、nominal definition tree の子型と function signature 内の duplicate structural / nominal type は、stable key が同じなら同じ論理型として扱う。これにより definition-side `.T` が instantiated `.T` を shadow する false miss と、同じ stable nominal identity を持つ signature duplicate の false miss を避ける。

同 checkpoint の release Web RPN same-session code edit 測定では、初回 `compile_ms=9870`、2 回目 `compile_ms=8512` だった。raw-init param facts は初回 `stores=163`、2 回目 `hits=142` / `resource_summary_value_replay_hits=142` まで伸び、`raw_init_param_facts_reprojection_context_bypasses` は `0` になった。残件は `raw_init_param_facts_reprojection_value_bypasses=25` と `raw_init_param_facts_incomplete_leaf_bypasses=37` である。前者は value replay の projection / type canonicalization issue、後者は byte-range / variant / return facts の complete mirror issue として分離する。

2026-05-31 の raw-init value reprojection checkpoint では、value replay 失敗を context 構築失敗からさらに分け、param cell projection / param cell type / release requirement projection / release requirement type の counter として観測できるようにした。raw address 上の `Deref` は通常の reference dereference ではなく raw cell view なので、raw-init param cell に限って stable entry が持つ cell 型を再投影先として使う。field / tuple / enum payload の layout 検証は従来どおり維持し、corrupt offset は projection mismatch として拒否する。

同 checkpoint の release Web RPN same-session code edit 測定では、初回 `compile_ms=9358`、2 回目 `compile_ms=1` だった。raw-init param facts は初回 `stores=165`、`raw_init_param_facts_reprojection_value_bypasses=23` になり、内訳は `param_cell_projection=0`、`param_cell_stable_type=23`、release 系 `0` だった。non-signature nominal value type は現在の `TypeCtx` 内 stable key から再投影できるようにしたが、boundary 外 labelled open generic は同名衝突を stable key だけで解決できないため fail-closed のまま残す。

owner boundary checkpoint では、`owner_summary_type_params` が signature/result に加えて raw memory load/store/fill の value type、user call type arguments、indirect call signature、collection slot lifecycle/drop/transform の value type を収集するようにした。raw-init param facts の value type は callee summary や collection slot proof の内側にだけ現れることがあるため、summary replay の意味に関わる型だけを owner summary boundary へ昇格し、cache key の type boundary hash と `ResourceSummaryTypeReprojection` の strict duplicate check に通す。単なる local type や TypeCtx 全体検索は authority にしない。

同 checkpoint の release Web RPN same-session 測定では、初回 `compile_ms=9615`、`stores=165`、`bypasses=60`、`incomplete_leaf=37`、`reprojection_value=23`、`param_cell_stable_type=23` であり、数値改善はまだ出ていない。この結果から、残る 23 件は単純な boundary 収集漏れではなく、同名 labelled generic の provenance / ordinal を stable entry と key に持たせる必要がある。`var(T:...)` だけで現在 `TypeCtx` から再対応付けする緩和は stale hit の危険があるため採用しない。

2026-05-31 の projection-derived raw-init replay checkpoint では、`param_cell_stable_type=23` の根本原因をさらに分解した。raw-init param cell の型が base parameter と suffix から通常の layout 規則で決まる場合、その型は現在 compile の signature と projection から再計算する。stable entry に保存された cell 型は、raw address `Deref` のように typed projection だけでは値型を得られない場合の proof boundary に限定する。この境界により、同名 labelled generic を TypeCtx 全体検索で拾う緩和を入れず、projection-derived value type の false miss だけを取り除く。

同 checkpoint では、direct user call の `type_args` も raw-init summary replay に渡すようにした。param / return / release / variant summary 内の型は、callee summary の `type_params` と call site の `type_args` から instantiation してから現在 compile の place suffix と照合する。これにより、summary replay に必要な generic substitution を value cache の stable mirror 境界で明示し、後段 Resource IR proof へ未置換の callee-local open generic を漏らさない。

release Web RPN same-session code edit 測定 `tmp/rpn_projection_authoritative_raw_init_type_20260531.json` では、初回 `stores=188`、`bypasses=37`、`incomplete_leaf=37`、`reprojection_value=0`、`param_cell_stable_type=0`、`param_cell_result_type=0` だった。2 回目は `raw_init_param_facts_hits=144`、`resource_summary_value_replay_hits=167`、`replayed_ops=167`、`recomputed_ops=21` で、value reprojection 残件は解消した。残る 37 件は byte-range / variant / return facts を含む complete raw-init mirror 不足であり、`ISS-20260528T123956163Z-RESOURCE-SUMMARY-RAW-INIT-CACHE-NEED-245DC1A5` へ継続する。

2026-05-31 の complete raw-init leaf mirror checkpoint では、`RawCellInitializationFunctionSummary` の `return_cells` / `return_byte_ranges` / `param_cells` / `param_byte_ranges` / `param_release_requirements` / `variant_param_cells` / `variant_param_byte_ranges` / `variant_required_param_cells` / `variant_conditions` を同じ stable entry に保存するようにした。これにより partial raw-init summary を保存して proof を欠落させる経路を避けつつ、byte-range / variant / return facts を replay 対象へ含める。

entry kind と内部 API 名は complete leaf へ改めた。一方で、`loader_cache_stats_json` の `raw_init_param_facts_*` counter 名は既存測定 JSON との互換のため維持している。現時点では historical metric name であり、実際の cache value は complete raw-init leaf entry である。

RPN same-session code edit 測定 `tmp/rpn_complete_raw_init_mirror_code_edit_20260531.json` では、comment-only edit は compiled-output cache により `compile_ms=1` になった。raw-init replay 評価用に `main` へ local `i32` binding を追加した code edit では compiled-output cache が miss し、base `compile_ms=8677`、edit `compile_ms=6586` だった。edit delta は `raw_init_param_facts_hits=205`、`resource_summary_value_replayed_ops=238`、`resource_summary_value_recomputed_ops=36`、`raw_init_param_facts_incomplete_leaf_bypasses=0` である。`incomplete_leaf` の根本原因は解消したが、edit delta で `raw_init_param_facts_reprojection_value_bypasses=15`、`param_cell_result_type=15` が残る。これは complete mirror 不足ではなく return / byte-range / variant surface の型 canonicalization 残件であり、`ISS-20260531T132755602Z-RAW-INIT-COMPLETE-LEAF-REPROJECTION-TYPE-CANON-4E8A1A2C` へ分離する。

return / byte-range type canonicalization checkpoint では、place projection 側も param cell と同じ replay authority 規則に揃えた。通常の layout projection から型が決まる場合は、保存済み stable type key ではなく現在 compile の function result/signature と suffix から型を再計算する。保存済み stable type key は final raw `Deref` のように typed projection だけでは値型を得られない場合にだけ proof boundary として照合する。

RPN same-session code edit 測定 `tmp/rpn_return_type_canonicalization_code_edit_20260531.json` では、base `compile_ms=8861`、edit `compile_ms=6703` だった。edit delta は `raw_init_param_facts_hits=205`、`resource_summary_value_replayed_ops=253`、`resource_summary_value_recomputed_ops=21`、`raw_init_param_facts_bypasses=0`、`raw_init_param_facts_reprojection_value_bypasses=0`、`param_cell_result_type=0` である。raw-init complete leaf replay の false miss は解消したが、compile time はまだ秒単位であり、次は replay 後にも残る `recomputed_ops=21` と Resource IR summary 外の固定費を分解する必要がある。

map key は namespace hash、function identity、function body hash、type parameter boundary hash、generic type argument hash、source capability policy hash、summary kind/version を含む。map value は summary kind ごとの stable entry だけであり、`ResourceFunction`、Resource IR body、既存 summary struct 全体、`TypeId`、`Span`、`SourceMap`、typed HIR、diagnostic span、raw alias graph、`RawCellInitializationFunctionSummary` 全体は保存しない。同じ関数内に複数の top-level summary fact が現れた場合は、entry 内で順序と重複を保持する。

store/hit 記録は summary build pass の全 candidate を集めてから行う。同じ compile pass 内で先に store した value を即 hit と数えると、微小変更時の再利用可能性を過大評価するため、hit 判定は記録開始時点で map に存在した value に限定する。`clear_loader_cache()` は loader cache、compiled-output cache、prewarm surface と同じ寿命境界で Resource summary value cache の map と stats も消す。

2026-05-28 の i32 scalar summary local reuse checkpoint では、Resource summary cache の大きい設計へ進む前の局所削減として、`compute_i32_scalar_return_summaries` の relevance 判定で `I32LeafProjectionCache` を共有し、複数 block 関数だけ `initial_i32_scalar_path_state` を関数内で 1 回構築して block ごとに clone するようにした。1 block 関数では clone 分が増えるため従来どおりその場で構築する。さらに return fact merge は path が 1 本の場合、全 path に対する包含確認を省き、同一 path 内の重複除去だけを行う。これは sibling enum variant をまたぐ merge 条件を緩める変更ではなく、複数 path が存在しない場合の同値な fast path である。

同 checkpoint の native release RPN stage-only 測定は、`resource_initialized_i32_scalar_summaries=1568ms`、`resource_initialized_raw_init_summaries=2705ms`、`resource_initialized_function_checks=1994ms`、`resource_static_check=7299ms` だった。前 checkpoint の `resource_static_check=7841ms` からは改善したが、初回 compile 0.5 秒未満にはまだ到達していない。`resource_ir` integration test は、既存 test fixture が `CollectionSlotLifecycleEvent::StorageDealloc { value_ty }` へ追従していないため compile error で実行できず、今回の i32 scalar summary 変更とは別の test drift として扱う。

必須 regression:

- 同じ entry source の 2 回目 compile は compiled-output cache hit として観測される。
- entry body-only edit で compiled-output cache は miss するが、unchanged stdlib の `DropTraversal` / `ForallInitializedRange` stable summary は hit する。
- public callable type edit、dependency public surface hash edit、generic type-argument edit、source capability policy edit、target/profile edit では stale hit しない。
- `/stdlib` overlay、forced stdlib VFS、local stdlib override では bundled stdlib Resource summary value cache を bypass する。
- cache hit value を現在 compile の Resource IR context へ再投影できない場合は miss/bypass に倒し、diagnostic span や capability proof を古い source へ流用しない。

## Empty source capability policy checkpoint

2026-05-31 の raw-init residual recomputation 調査では、`ResourceSummaryValueCache` の value replay は fail-closed に動いている一方で、key に含まれる source capability policy が file 全体の source hash に依存していたため、capability proof を持たない通常 source の小さな編集でも dependency closure が広く miss することを確認した。

source capability proof が存在する file は、proof span が byte range であるため、canonical path、source hash、proof set を合わせて policy hash とする必要がある。この境界を弱めると、別 source の同じ byte range に raw memory / collection slot authority を誤って再利用できる。

一方で、proof set が空の file では、source capability policy が保護すべき privilege surface は存在しない。関数の意味変更は Resource IR body hash、typed signature/type boundary、dependency closure hash で検出するため、空 proof file の source text 全体を source capability policy に混ぜると、静的検査の正確性を増やさずに over-invalidation だけを増やす。

この checkpoint では、`SourceCapabilities::stable_policy_hash` を次の境界にした。

- proof set が空でない file は、path、source hash、proof set を hash する。
- proof set が空の file は、path と空 proof set だけを hash する。
- raw body text や exact capability use-site を持つ file は従来どおり source hash に結び、stale hit を避ける。

release WASM の RPN same-session code edit 測定 `tmp/rpn_empty_source_policy_raw_init_code_edit_20260531.json` では、edit compile が直前の raw alias checkpoint `7142ms` から `6164ms` へ改善した。edit delta は次の通りである。

- `resource_raw_init_summary_recomputations=73`、直前の `81` から減少。
- `resource_summary_value_raw_init_param_facts_stores=48`、直前の `69` から減少。
- `resource_initialized_function_checks=1`、直前の `13` から減少。
- `resource_summary_value_recomputed_ops=29`、直前の `110` から減少。
- raw-init replay bypass は `0` のままで、unsafe な stale replay は観測されていない。

ただし、これは function-local exact source capability policy の完成ではない。capability proof を持つ stdlib/compiler-owned source では、source text と exact proof を結び付ける必要が残る。次段階では、関数本文 slice、相対 use-site identity、capability kind、raw body source slice を組み合わせた function-local policy hash を設計し、同一 file の sibling function edit が無関係な raw-init dependency closure を miss させないようにする。

同日の次 checkpoint では、この function-local policy hash を `ResourceSummaryValueCacheContext` に接続した。context は compile-local に source path、source text、`SourceCapabilities` を保持し、`ResourceFunction` の function/block/op/terminator span から file ごとの最小 scope を作る。scope 内に capability proof がある場合だけ、その source slice と相対 use-site span を hash し、scope 外の sibling source text は key に混ぜない。scope と capability proof が部分的に重なる場合は `None` に倒し、summary cache を store/replay しない。

さらに raw-init complete leaf cache は、fact を持たない relevant function も empty entry として保存するようにした。raw-init summary が空であることも fixed-point の結果であり、これを保存しないと no-fact function が微小編集ごとに worklist へ戻るためである。empty entry は現在の function boundary へ再投影できる場合だけ worklist skip として使い、下流の summary index へ空 summary は渡さない。これにより、従来「summary なし」だった呼び出しの意味を変えずに recomputation だけを削る。

`tmp/rpn_function_local_policy_empty_raw_init_filtered_code_edit_20260531.json` では、same-session code edit の `resource_raw_init_summary_recomputations` が `73` から `0` になった。raw-init replay bypass は引き続き `0` であり、下流へ渡す `resource_raw_init_summary_count` も `78` のまま維持した。一方で edit compile は `6105ms` でまだ秒単位であり、raw-init fixed-point は支配項ではなくなった。次は `resource_raw_alias_summary_recomputations=32`、raw alias residual reprojection、typed expression subtree query、codegen fragment cache を分けて進める。

同日の raw alias residual checkpoint では、`raw_alias_return_entry` の value reprojection 失敗を
reason 別 counter へ分解し、projection で型が決まった後に保存済み stable type key を再度
`TypeId` へ戻す段階が残件であることを確認した。修正では raw-init complete leaf replay と同じく、
通常 projection の結果型は現在 compile の function signature / suffix を replay authority とし、
raw address 由来の終端 `Deref` だけ保存済み stable type key を proof boundary として使う。
途中 `Deref` 後に field / storage offset などが続く場合は、後続 layout を検証できないため
fail-closed のまま拒否する。

`tmp/rpn_raw_alias_projection_type_replay_code_edit_20260531.json` では、
`resource_summary_value_raw_alias_return_entry_reprojection_value_bypasses=0`、
`resource_summary_value_raw_alias_return_entry_bypasses=0`、
`resource_raw_alias_summary_recomputations=1` になった。残る 1 件は編集した関数自身の再計算であり、
raw alias summary の false miss は支配項から外れた。edit compile は `5923ms` で、次は typed
expression subtree query、codegen fragment cache、変更関数自身の Resource summary 再計算、
Resource IR summary 外の固定費を分けて進める。

同日の stage timing JSON checkpoint では、Web / Node の `CompilerSession.loader_cache_stats_json()` に
直近 compile の stage timing を追加した。native の `NEPL_COMPILE_STAGE_TIMING=1` は標準エラーへ
出力するだけなので、playground / Node test runner では compiled-output cache miss 後の支配 stage を
JSON artifact として残せなかったためである。cache hit では `compile_stage_timings=[]` を返し、
real compile では target precheck から wasm validation までの粗い stage 配列を返す。
`compile_stage_timing_status` は `not_started`、`cache_hit`、`compiled`、`failed` のいずれかで、
`[]` が cache hit なのか stage に到達しない早期失敗なのかを区別する。Web / Node では
`performance.now()` を優先し、存在しない host だけ `Date.now()` に fallback する。

`tmp/rpn_stage_timing_same_session_code_edit_20260531.json` では、同一 `CompilerSession` で RPN を
一度 compile した後、`main` 内の表示用 string literal だけを変更した。base は
`compile_ms=12549`、edit は `compile_ms=5779` で、edit の stage timing は次の通りである。

| stage | elapsed |
|---|---:|
| `resource_typecheck` | 154ms |
| `resource_monomorphize` | 5ms |
| `resource_static_check` | 5492ms |
| `codegen_precheck` | 5ms |
| `wasm_codegen` | 19ms |
| `wasm_validate` | 2ms |

同 edit の Resource summary delta は `resource_summary_value_replayed_ops=4553`、
`resource_summary_value_recomputed_ops=16`、`resource_raw_alias_summary_recomputations=0`、
`resource_raw_init_summary_recomputations=0`、`resource_initialized_function_checks=0` だった。
これにより、現時点の秒単位コストは raw-init / raw-alias / final check の false miss ではなく、
大量の proof replay と Resource static check pipeline の固定費として残っていることが分かった。
次段階は typed expression subtree query と binary intermediate artifact により、変更されていない
stdlib / dependency proof と codegen fragment を session または disk artifact から直接差し替える。

### Final Check Lazy Pass Replay

2026-05-31 の final initialized check lazy pass checkpoint では、cache hit した
final initialized function check を、final cell / collection slot state の materialized
replay ではなく、diagnostic-free / auto-drop-free な checked pass として戻すようにした。
保存時点で diagnostics と auto drop points を持つ関数は no-store に倒しているため、hit
した entry は cell gate と drop elaboration が必要とする surface だけで pass として扱える。

この変更では、後続 stage が現在消費しない `final_cells` / `final_collection_slots` を
現在 `TypeCtx` / `Place` へ再投影しない。将来 final state を読む stage を追加する場合は、
lazy pass ではなく明示的な materialized replay API を使う。cache hit の安全条件は
`initialized_function_check_entry_key` の body hash、dependency closure、type boundary、
source capability policy に残し、不要な proof value 復元だけを削る。

同時に、`ResourceSummaryValueCacheContext` は function source capability policy hash を
compile-local に memoize する。summary kind ごとに同じ関数へ繰り返す純粋 query を
避けるためであり、key は name / origin name / function span を含めて同名関数の誤共有を
避ける。

release Web RPN same-session string literal edit 測定
`tmp/rpn_lazy_initialized_check_code_edit_20260531.json` では、base `compile_ms=11612`、
edit `compile_ms=5454` だった。直前の同種測定
`tmp/rpn_stage_timing_same_session_code_edit_20260531.json` の edit `compile_ms=5779` /
`resource_static_check=5492ms` に対し、今回の edit は `resource_static_check=5170ms`
である。

edit の累積差分は、`resource_raw_alias_summary_recomputations=0`、
`resource_raw_init_summary_recomputations=0`、`resource_initialized_function_checks=0` を
維持しながら、`resource_summary_value_lazy_pass_hits=288`、
`resource_summary_value_lazy_pass_ops=3639` になった。materialized proof replay は raw alias /
i32 scalar / raw-init 側の `resource_summary_value_replayed_ops=914` まで縮小しており、
final check 由来の全関数 final state 復元は支配項から外れた。

ただし edit compile はまだ 0.5 秒未満に届いていない。残りは Resource static check
pipeline の固定費、stage-local key / dependency hash の重複構築、changed function only
proof replay、typed expression subtree query に分解して継続する。

### Dependency Closure Base Hash Table

2026-05-31 の dependency closure base hash checkpoint では、Resource summary value
cache の dependency closure hash を「summary kind tag」と「kind に依存しない closure
base hash」に分けた。closure base hash は、reachable dependency closure の function
identity、Resource IR body hash、source capability policy hash、function-local type
boundary を含む。これは stale hit を避けるための invalidation 入力であり、今回の変更で
削除していない。

同じ Resource static check stage 内では、raw alias / i32 scalar / raw-init / final
initialized check が同じ dependency graph と function index に対して同じ closure を
繰り返し走査する。そこで `ResourceSummaryValueCacheContext` に compile-local な table を
置き、module functions slice、dependency graph slice、function index を key として
closure base hash を再利用する。table key は in-memory pointer identity を含むため、
永続 artifact には保存しない。source capability policy input が追加・更新された場合は、
function source policy cache と同時に dependency closure base hash table も clear する。

summary kind tag は base hash の外側で `raw-alias`、`raw-init`、`i32-scalar`、
`initialized-function-check` ごとに重ねる。これにより、同じ closure base を共有しても
summary kind 間で cache key が衝突しない。

release Web RPN same-session string literal edit 測定
`tmp/rpn_dependency_closure_base_cache_20260531.json` では、base `compile_ms=9431`、
edit `compile_ms=3138` だった。edit の `resource_static_check` は `2833ms` で、
lazy final check checkpoint の `5170ms` から大きく下がった。

この測定の session 統計は累積値であり、base から edit への差分では
`resource_raw_alias_summary_recomputations=0`、`resource_raw_init_summary_recomputations=0`、
`resource_initialized_function_checks=0` を維持している。`resource_i32_scalar_summary_recomputations`
だけ `+7`、`resource_summary_value_recomputed_ops` は `+16` であり、支配点は false miss
ではなく Resource static check pipeline の未分割固定費へ移っている。

まだ 0.5 秒未満には届いていないため、次段階では changed function only proof replay、
typed expression subtree query、stdlib prechecked `.neplmeta`、function-level `.neplobj`
を順に進める。

### Skip Re-recording Preseeded Summary Entries

2026-05-31 の preseeded summary record skip checkpoint では、Resource summary cache から
worklist 前に replay 済みの entry を、同じ compile の末尾で candidate として再記録しない
ようにした。replay 時点で key 作成、stable entry の存在確認、現在の type / place boundary
への fail-closed な再投影が完了しているため、末尾の candidate 化は安全性 proof を強めず、
同じ key / dependency closure / stable mirror をもう一度構築する固定費だけを増やしていた。

適用対象は raw alias、i32 scalar、raw-init complete leaf、collection slot complete leaf
である。ただし、preseed された関数が依存先変更により同じ fixed-point pass 内で再び
worklist に入った場合は、再計算済みの新しい summary として通常どおり candidate 化する。
この境界は `SummaryWorklist::unrecomputed_initial_skips` で表し、「初期 skip されたまま
一度も再計算されなかった関数」だけを record 対象から外す。

この変更により、`resource_summary_value_hits` と summary kind 別 hits は「通常 recompute
後に candidate 化した entry が既存 stable value と一致した数」を表す。preseed replay に
よって実際に compile work を削った量は、引き続き `resource_summary_value_replay_hits` /
`resource_summary_value_replayed_ops` / `resource_summary_value_lazy_pass_*` を見る。

release Web RPN same-session string literal edit 測定
`tmp/rpn_skip_preseeded_summary_record_20260531.json` では、base `compile_ms=9403`、
edit `compile_ms=2105` だった。edit の `resource_static_check` は `1820ms` で、直前の
dependency closure base hash checkpoint の `2833ms` からさらに下がった。base から edit
への差分では、`resource_raw_alias_summary_recomputations=0`、
`resource_raw_init_summary_recomputations=0`、`resource_initialized_function_checks=0` を維持し、
`resource_i32_scalar_summary_recomputations=+7`、`resource_summary_value_recomputed_ops=+16`
が残る。preseed 再記録分を数えなくなったため、edit 累積の
`resource_summary_value_hits` と kind 別 hits は `0` で、実 reuse は
`resource_summary_value_replayed_ops=914` と `resource_summary_value_lazy_pass_hits=288` に現れる。

0.5 秒未満にはまだ届いていない。残る支配項は Resource static check pipeline の
changed-function-only 化、typed expression subtree query、stdlib prechecked artifact、
codegen fragment cache に分けて継続する。

### Recomputed Ops Kind Counters

2026-05-31 の recomputed ops kind counter checkpoint では、aggregate
`resource_summary_value_recomputed_ops` を raw alias / i32 scalar / raw-init /
collection slot の summary kind 別 counter へ分けた。これは safety proof には影響しない
観測専用の変更であり、既存の aggregate counter は維持する。

`tmp/rpn_recomputed_ops_kind_counters_20260531.json` では、same-session string literal edit の
base から edit への差分として、`resource_summary_value_recomputed_ops=+16` がすべて
`resource_summary_value_i32_scalar_return_facts_recomputed_ops=+16` に属することを確認した。
raw alias、raw-init、drop traversal の kind 別 recomputed-op 差分は `0` である。同じ edit
では i32 scalar の `resource_summary_value_i32_scalar_return_facts_bypasses=+16` と
`resource_summary_value_i32_scalar_return_facts_reprojection_value_bypasses=+16` も増えるため、
残差は i32 scalar stable mirror の再投影失敗として扱う。

この結果に基づき、i32 scalar residual は
`ISS-20260531T134951396Z-I32-SCALAR-RESIDUAL-REPROJECTION-STI-0F6F5A24` へ分離した。
次に必要なのは entry / function / reason 単位 counter であり、changed-function-only proof
replay の scope 設計とは混ぜない。Resource static check 全体の秒単位コストは引き続き
全関数 replay / record / dependency preseed の固定費として、親 issue で追跡する。

### I32 Scalar Residual Reason Counters

2026-05-31 の i32 scalar residual reason counter checkpoint では、i32 scalar の残差を
replay 前の function count と、recompute 後の fact count に分けた。既存の
`resource_summary_value_i32_scalar_return_facts_recomputed_ops` は facts 数の aggregate として
維持し、entry 取得前に起きる replay miss は function count の別 counter にする。

追加した counter は、candidate store miss、candidate unstable entry の詳細、
candidate reprojection value の詳細、replay entry miss、replay reprojection value の詳細である。
これにより、同じ `+16` の残差が「entry が見つからない」のか「recompute 後の stable entry 化に
失敗している」のかを分けられる。

release Web RPN same-session string literal edit 測定
`tmp/rpn_i32_residual_reason_counters_20260531.json` では、base `compile_ms=8932`、
edit `compile_ms=2102` だった。edit の `resource_static_check` は `1816.135ms` である。
base から edit への差分は次の通りだった。

- `resource_summary_value_i32_scalar_return_facts_recomputed_ops=+16`
- `resource_summary_value_i32_scalar_return_facts_reprojection_value_bypasses=+16`
- `resource_summary_value_i32_scalar_return_facts_reprojection_value_scalar_type_bypasses=+10`
- `resource_summary_value_i32_scalar_return_facts_reprojection_value_parameter_projection_bypasses=+6`
- `resource_summary_value_i32_scalar_return_facts_reprojection_value_return_projection_bypasses=0`
- `resource_summary_value_i32_scalar_return_facts_replay_entry_miss_functions=+7`
- `resource_summary_value_i32_scalar_return_facts_misses=0`

この結果から、RPN edit の i32 scalar 残差は 7 関数の replay entry miss から再計算へ戻り、
再計算後の 16 facts が stable entry として保存できていないことが分かった。したがって次の
根本修正は key を弱めることではなく、`ParameterProjection` と `ScalarType` の stable mirror
再投影 surface を広げることである。

## 次段階の CompilerSession 設計

`CompilerSession` は、純粋な compiler query を process 内で保持する単位である。CLI では 1 process 1 session、Web / Node test runner では WASM instance 1 session とする。

session が持つ query cache は次の階層に分ける。

| query | key | value | invalidation |
|---|---|---|---|
| source text | canonical path + content hash | UTF-8 source | file / VFS overlay change |
| lex | source hash + lexer version | token stream | source text change |
| parse | token hash + parser version + type arity hint hash | AST module | source text or imported type arity change |
| import graph | canonical path + import directive hash | dependency edges / reverse edges | imported module public surface change |
| type arity | module public type decl hash | type arity table | public type declaration change |
| name/typecheck | module hash + dependency public surface hash + target/profile | typed HIR / diagnostics / trait table | dependency public surface or local source change |
| monomorphize | typed HIR hash + instantiation root set | monomorphized HIR | reachable root or generic instantiation change |
| resource summary | namespace key + function body hash + generic type-argument hash + source capability policy hash + summary kind/version | arena 非依存 stable mirror summary | public surface / function body / type argument / capability / target/profile change |
| codegen | monomorphized reachable HIR hash + target/profile | wasm / llvm fragment | reachable lowered HIR change |

各 query は入力値だけで決まり、FileSystem や StdIO を内部で読まない。host 依存の file read は CLI / Web wrapper 側で source text table に変換してから session へ渡す。

## stdlib prechecked artifact

stdlib artifact は、release build または test runner startup で作成する。

artifact に含めるもの:

- bundled stdlib source hash table。
- module import graph と reverse dependency graph。
- public type / trait / function signature table。
- generic function の body hash と type parameter/kind boundary。
- trait impl index と method signature compatibility result。
- Resource IR summary template。generic type substitution が必要な部分は template として残す。
- source capability table。capability span は source hash と path に結び、古い source へ流用しない。

通常 program compile では、stdlib module の parse / import / signature / trait impl / capability validation を再実行しない。entry source と overlay source だけを新規 query として処理し、stdlib generic instantiation と reachable codegen fragment だけを必要に応じて具体化する。

## incremental compile

微小変更では、changed source hash から reverse import graph を辿り、影響を受けた query だけを invalidation する。

### 式枝差し替えの 0.1 秒 budget

リテラルの書き換えは、差分コンパイル設計では特別扱いしない。リテラルも式木の leaf であるため、目標は「typed AST / HIR のある式枝を、同じ公開境界を持つ別の式枝へ差し替える」操作全般を同一の incremental query として扱うことである。

同一 `CompilerSession` 内で、次の条件を満たす式枝差し替えは 0.1 秒以下を目標にする。

- 変更範囲が 1 function body または小さい user module 内の式枝に閉じる。
- module の public type / trait / function signature、import / export surface、source capability policy が変わらない。
- 差し替え後の式枝が既存の名前解決・型推論境界で解決でき、dependency public surface を変えない。
- static call graph の conservative-all fallback を起動しない。
- stdlib artifact と bundled stdlib hash が current session と一致している。

この budget では、source text 全体を key にした compiled-output cache だけでは足りない。compiled-output cache は完全同一またはコメントだけの変更を即時返す境界であり、実コードの式枝差し替えでは miss する。そのため、次の query 境界を function / expression subtree 単位へ分ける。

| query | key | value | 式枝差し替え時の扱い |
|---|---|---|---|
| lex / parse | source hash + parser version | AST module | 変更 source だけ再実行する。 |
| name surface | module public surface hash | public decl table | public surface 不変なら依存 module は invalidation しない。 |
| typed expr subtree | function identity + lexical path id + subtree semantic hash + expected type boundary | typed expression / diagnostics | 差し替えた枝と、その expected type / local name scope に依存する枝だけ再型検査する。 |
| function body HIR | function identity + body semantic hash + local binding shape hash | typed HIR body | local binding shape が変わらない枝差し替えは unchanged block / sibling expression を reuse する。 |
| Resource IR function | typed HIR function hash + source capability policy hash | ResourceFunction | 変更 function だけ lowering し、他 function の Resource summary key は body hash で hit させる。 |
| Resource summary | namespace + function body hash + dependency closure hash + source policy + summary kind | stable mirror summary | 変更 function とその summary dependents だけ再計算し、他は preseed する。 |
| codegen fragment | monomorphized function identity + lowered body hash + target/profile | wasm / llvm fragment | 変更 function fragment だけ再生成し、table/signature/link order を再接続する。 |

`lexical path id` は span の byte offset そのものではなく、関数内の AST path と local binding shape から作る stable id にする。単純なリテラル長変更や前方へのコメント追加で id が揺れると 0.1 秒 budget を満たせないためである。一方で、local binding の追加・削除、pattern shape、capture / scope boundary が変わる場合は、その scope の descendant query を invalidation する。

型推論は expected type を前置・外側から受け取れる場合が多いが、NEPLg2 の prefix call reduction は callable candidate / arity / expected type を使って式境界を解決する。したがって、typed expr subtree query は「source substring だけ」ではなく、local name scope、expected type、callable candidate set、generic type argument mode、effect expectation を key に含める。これらを省くと、同じ式 text が別 context で違う型や effect になる stale hit を起こす。

Resource IR 以降は、式枝差し替えを function body hash の変更として扱う。変更 function の final check / raw alias / i32 scalar / raw-init / collection slot summary は必要に応じて再計算し、dependency closure hash により dependent function だけを再投入する。既存の Resource summary value cache はこの境界の下位実装であり、0.1 秒 budget のためには raw alias summary と raw-init residual recomputation も stable preseed へ載せる必要がある。

0.1 秒 budget の対象外:

- stdlib source、public signature、trait impl surface、source capability use-site を変更する場合。
- local binding shape や scope graph が大きく変わり、式枝の stable path 対応が失われる場合。
- indirect call / raw body / unresolved function value により call graph が閉じず conservative-all になる場合。
- diagnostics の source span を現在 source map へ安全に戻せない場合。

MVP では次の順に実装する。

1. Web / Node に `CompilerSession` API を追加し、bundled stdlib source table を保持する。
2. Web terminal は compile ごとに worker を破棄せず、明示的な artifact refresh まで同一 worker / WASM instance / `CompilerSession` を維持する。
3. `CompilerSession` に warm parsed stdlib module cache を追加し、entry source が変わっても stdlib parse/import/type arity/typecheck artifact を再利用する。
4. Resource IR summary stable mirror を function hash 単位で cache し、entry から到達する changed functions だけを再計算する。MVP では `DropTraversal` / `ForallInitializedRange` から始め、raw initialization summary 全体は store しない。
5. codegen fragment を function hash 単位にし、unchanged functions は index と signature table だけを再接続する。
6. diagnostic rendering は最後に行い、cache には typed diagnostic enum と source span だけを保持する。

10ms 未満の対象は、CompilerSession が warm で、stdlib artifact が current で、変更が entry source の一部または小さな user module に閉じる場合である。stdlib 自体を変更した直後は artifact refresh が必要なので、この budget の対象外とする。

## binary intermediate artifact

現状の NEPLg2 は、JVM の `.class` や C 系の `.o` に相当する永続的な binary intermediate artifact をまだ作っていない。存在するのは process-local な `LoaderSessionCache`、`ResourceSummaryValueCache`、`CompilerSession` の compiled-output cache、そして最終 wasm / WAT 補助情報である。`PreparedProgram` は typed HIR、型コンテキスト、Resource drop plan、public signature を持つが、現時点では memory 上の中間値であり、session をまたいで読み書きできる object file ではない。

差分コンパイルの仕様では、`.class` 型の「検査済み typed object」と `.o` 型の「target-specific relocatable fragment」を分離する。NEPL の静的検査は Resource IR、effect、source capability、private effect mask proof に依存するため、単に wasm fragment だけを保存しても安全性 proof の再利用条件を説明できない。逆に typed HIR だけでは 10ms 級の最終 output 差し替えに届かないため、backend fragment と link manifest も必要である。

### current design と提案 design の統合

現在進めている設計は、まず同一 `CompilerSession` 内で `LoaderSessionCache` と
`ResourceSummaryValueCache` を増やし、永続ファイルへ出す前に stale hit しない
stable key / stable mirror / fail-closed replay を固める方針である。この方針は実装
リスクが低く、Web playground の秒単位停止を直接削れる。一方で、session をまたいだ
reuse はできず、現状の RPN code edit のように cache hit 後の全関数 proof replay が
残る。

新提案の 4 層設計は、interface、typed HIR、Resource proof、backend object を分ける
点が正しい。NEPLg2 は prefix call boundary を型検査と callable candidate 解決に依存
するため、`.o` 相当だけでは依存 module の再解析・再型検査を十分に減らせない。最初に
public interface artifact を固定し、その上へ typed / proof / backend artifact を積む
必要がある。ただし、typed HIR は `TypeId`、`DefId`、`FileId`、`Span` を含みやすく、
現状の Rust 実装の値をそのまま disk へ保存してはいけない。

統合した方針は次の通りである。

- 短期: process-local cache で key と stable mirror の正確性を固める。
- 中期: `.neplmeta` と `.neplproof` を disk-backed artifact にする。
- 長期: stable lexical path id / stable typed id が揃ってから `.neplhir` を永続化する。
- backend: Resource / typed artifact が十分に効いた後で `.neplobj` を入れる。

### artifact stack

拡張子は NEPL artifact であることが分かる `.nepl...` 形に統一する。短い `.nei`、
`.ners`、`.neo` は役割を表しにくく、他の toolchain artifact とも衝突しやすいため
採用しない。

| artifact | 役割 | 主な key |
|---|---|---|
| `.neplmeta` | import graph、exported type/function/trait impl surface、effect signature、typed public signature、source capability policy surface を保持する。依存側の名前解決・型検査がまず読む authority である。 | module path、source key hash、source public surface hash、typed public signature hash、dependency public surface hash、stdlib hash、compiler schema version |
| `.neplhir` | stable lexical path id、typed HIR、typed diagnostics enum、local binding shape、expected type boundary を保持する。MVP では同一 session cache に限定し、永続化は stable typed id 導入後に行う。 | function identity、body semantic hash、scope shape hash、callable candidate set、effect expectation |
| `.neplproof` | Resource IR lowering 結果、initialized/owner/borrow/drop/effect summary、private effect mask proof を stable mirror として保持する。 | typed HIR function hash、source capability policy hash、type/effect boundary hash、generic type arguments、summary kind |
| `.neplobj` | wasm / LLVM の function body fragment、signature table entry、function table entry、data segment、relocation metadata を保持する。 | monomorphized function identity、lowered body hash、target/profile、backend feature set |
| `.nepllink` | symbol、relocation、table index、data offset、entry point、export/import を再接続する。`.neplobj` を最終 wasm / LLVM artifact へ組み立てる manifest である。 | dependency public surface hash、fragment set hash、target/profile |

この構造では、リテラル変更や同じ expected type への式枝差し替えは、まず typed expression subtree query を invalidation する。public surface と local binding shape が変わらないなら、依存 module の public surface object は再利用し、変更 function の typed object と Resource proof object だけを再計算する。codegen では変更 function fragment だけを再生成し、unchanged fragment は link manifest の再接続で使う。

stdlib は最初の対象にする。stdlib の public surface object、typed object、Resource proof object は release artifact に同梱し、user source の compile では generic type arguments の具体化と link だけを行う形へ寄せる。generic function は template object と specialization object を分け、type arguments、trait impl surface、effect boundary が一致する場合だけ specialization proof を再利用する。

binary intermediate artifact は cache hit のために safety proof を弱めてはならない。次のどれかを現在 compile の context へ再投影できない場合は、artifact を使わず再計算する。

- source capability exact use-site と source policy hash。
- generic type argument と trait impl surface。
- private effect mask proof と region non-escape proof。
- diagnostic span の stable lexical path id。
- function table / data segment relocation。
- compiler version、schema version、target/profile、stdlib content hash。

最初に実装する disk artifact は `.neplmeta` である。これは既存の public surface hash と
typed public signature table を外部化するだけなので、`TypeId` や `Span` を保存しない
既存境界と合っている。次に `.neplproof` を `ResourceSummaryValueCache` の stable mirror
から作る。`.neplhir` は便利だが、session-local identity を永続化しない設計が先に必要な
ため、最初は in-memory query cache として扱う。`.neplobj` / `.nepllink` は codegen の
支配時間が残った段階で導入する。

### 2026-06-01 `.neplproof` snapshot / preseed boundary

`.neplproof` の最初の実装単位として、`ResourceSummaryValueCache` に
`export_neplproof_snapshot` と `preseed_neplproof_snapshot` を追加した。この snapshot は
disk schema ではなく、disk-backed artifact へ出してよい stable payload を `core` 内で
明確にするための in-memory boundary である。

保存対象は `ResourceSummaryValueCacheKey` と stable mirror entry に限定する。現 checkpoint
では次を snapshot に含める。

- `CollectionSlotDropTraversalForallLeafEntryV1`
- `RawAliasReturnEntryV1`
- `I32ScalarReturnFactsEntryV1`
- `InitializedFunctionCheckEntryV1`
- `OwnerObligationCheckEntryV1`
- `RawInitCompleteLeafEntryV1`

一方で、`ResourceSummaryReplaySnapshot` と `InitializedFunctionCheckPassSnapshot` は
`.neplproof` payload に含めない。これらは前回 compile の関数順序と local fingerprint に
依存する changed-function plan であり、disk artifact の authority ではない。diagnostic span、
`TypeId`、`Span`、`SourceMap`、`ResourceId`、`StorageId` の生値も同じ理由で保存しない。

`preseed_neplproof_snapshot` は既存 cache entry を上書きしない。同じ key で同じ value なら
既存一致として数え、同じ key で異なる value があれば conflict として拒否する。preseed 後に
実際に proof を使うかどうかは、従来の replay API が現在の `TypeCtx`、function signature、
source capability policy、generic boundary へ再投影できるかで決める。したがって、この段階は
「cache を読む権利」を与えるだけであり、Resource proof 自体の検査を省略する authority では
ない。

同日の次 checkpoint では、snapshot に対して `ResourceSummaryProofArtifactHeader` と
`ResourceSummaryProofArtifact` を追加した。これは `.neplproof` の実 serialization ではなく、
disk / IndexedDB / bundled stdlib artifact を cache に preseed する前の envelope である。
header は schema version、compiler identity hash、target hash、profile hash、stdlib content
hash、dependency public surface hash、Resource summary namespace hash、source capability
policy set hash、private effect policy hash を分けて持つ。1 field でも現在 compile の期待
header と一致しない場合は payload を読まずに拒否する。

この envelope は `ResourceSummaryValueCacheKey` に含まれる per-entry invalidation を置き換えない。
artifact 全体の target / stdlib / dependency / policy 境界を先に検査し、個別 entry は従来どおり
replay 時に fail-closed に再投影する。これにより、`.neplproof` を stdlib release artifact として
同梱する場合でも、overlay stdlib、compiler schema 変更、private effect mask policy 変更を
artifact-level miss として扱える。

この分離により、Web playground の warm session は memory cache で同じ構造を使い、CLI / CI / selfhost compiler は disk-backed artifact として同じ invalidation rule を使える。selfhost 実装でも、純粋 query function の結果を private cache へ保存する設計と整合し、cache は外部観測可能な semantics ではなく compile-time acceleration として扱う。

同日の private cache exact boundary checkpoint では、`.neplproof` の `source capability policy set hash`
が守るべき use-site authority を Resource effect boundary gate にも接続した。`EffectOp::PrivateCache`
は `PrivateCacheOutsideBoundary` として一度 fail-closed に診断され、現在 compile の
`SourceMap::private_cache_boundary_allowed_at(span, operation)` が同一 file / exact span /
same operation を証明した場合だけ trusted use-site 診断を suppress する。この suppress は
`.neplproof` の stale-hit 防止境界と同じ入力に基づくが、`PrivateCacheInPureFunction` を
Pure へ mask する authority ではない。private region non-escape proof がない artifact は、
SourceCapability が一致しても pure memoization proof としては使わない。

同日の `.neplproof` compile-context header checkpoint では、expected header の生成を
`nepl-core` の compiler pipeline へ寄せた。`ResourceSummaryCacheNamespaceKey` は
typecheck 後に確定するため、Web / CLI 側で target/profile hash や resource summary namespace
hash を再実装すると stale-hit 防止境界が分裂する。そこで
`ResourceSummaryCacheNamespaceKey::resource_summary_proof_header` が compiler identity、
target/profile、stdlib content hash、dependency public surface hash、Resource summary namespace、
source capability policy set、private effect policy version から
`ResourceSummaryProofArtifactHeader` を作る。

`ResourceSummaryProofArtifactCacheOptions` は、host が持つ optional preseed artifact と
stdlib content hash だけを core compiler へ渡す。artifact は Resource static check の直前に
expected header と照合され、mismatch なら payload を cache へ merge せず通常 compile へ戻る。
private effect policy hash はこの path では常に `Some` にし、`None == None` による private
cache / mask rule の誤共有を避ける。disk / IndexedDB codec はまだ作らず、将来追加するときは
header decode / compare を payload decode より前に置く。

同日の Web `CompilerSession` checkpoint では、same-session の `.neplproof` artifact slot を
1 つ追加した。compiled-output cache miss の実 compile だけが、前回 artifact を
`ResourceSummaryProofArtifactCacheOptions` として core へ渡し、成功後に現在の
`ResourceSummaryValueCache` から新しい artifact を保存する。compiled-output cache hit では
core pipeline が走らず expected header も再計算されないため、cache entry に保存していた
artifact を session slot へ戻すだけにする。

stdlib overlay がある compile では loader cache と Resource summary value cache を bypass する
既存条件に合わせ、`.neplproof` preseed/export も行わない。bundled stdlib content hash は
`STD_LIB_HASH` の `fnv1a64:{hex}` suffix を Rust 側で `u64` に変換し、JS number / `f64` を
通さない。変換できない場合は artifact を作らず、同じ session の memory cache だけを使う。

この slot は disk / IndexedDB codec ではない。payload 本体を JS へ公開せず、
`loader_cache_stats_json` には artifact の有無、保存可能 entry 数、store/preseed candidate 数、
stdlib hash parse 可否だけを出す。`TypeId`、`Span`、`SourceMap`、diagnostic span、
changed-function replay plan は従来どおり `.neplproof` payload には含めない。

### 2026-05-31 measurement boundary

RPN same-session code edit は、Resource summary value cache の段階的な stable mirror により
秒単位ではあるが改善している。`tmp/rpn_i32_open_generic_reprojection_code_edit_20260531.json`
では、code string literal edit が `compile_ms=2126`、`resource_static_check=1841.527ms`
まで下がった。これは raw-init / raw-alias / final check の false miss を削り、i32 scalar
stable mirror の projection-derived type replay を一部修正した結果である。

ただし同じ測定の base compile は `compile_ms=9231`、`resource_static_check=8606.798ms`
である。warm edit path の改善は重要だが、NEPLg2 の性能目標は 1 program compile を
0.5 秒未満へ近づけることも含む。したがって、次の設計判断では edit cache hit の
counter だけを成功条件にしない。base compile では stdlib の parse / typecheck / Resource
proof template / codegen fragment を `.neplmeta` / `.neplproof` / `.neplobj` へ寄せ、初回から
「ほぼ link するだけ」に近い状態を作る必要がある。

### 2026-05-31 dependency graph sharing checkpoint

Resource static check 内の raw alias、i32 scalar、raw-init、collection slot、final
initialized check は、同じ `ResourceModule` の関数依存関係を使って固定点計算を行う。
これまでは summary kind ごとに caller -> callee の dependency、callee -> caller の
dependent、初期 worklist 順序を構築していた。今回の checkpoint では
`ResourceSummaryDependencyGraph` を Resource static check の compile-local view として
作り、各 summary kind が同じ依存グラフを共有するようにした。

この変更は cache key や proof boundary を弱めない。関数本体 hash、source capability
policy、typed boundary、generic type argument による stale hit 防止は従来どおり維持し、
重複した graph construction だけを削る。`ResourceSummaryDependencyGraph` は現在の
`ResourceModule` に閉じた一時 view であり、`.neplproof` などの永続 artifact には直接保存しない。

測定では `trunk build --release` 後の Web RPN same-session code edit
`tmp/rpn_dependency_graph_share_code_edit_20260531.json` が、base `compile_ms=9246` /
`resource_static_check=8193.197ms`、unused local 追加 edit `compile_ms=2135` /
`resource_static_check=1857.811ms` だった。edit delta は
`resource_raw_alias_summary_recomputations=+1`、`resource_i32_scalar_summary_recomputations=+5`、
`resource_raw_init_summary_recomputations=0`、`resource_initialized_function_checks=+1`、
`resource_summary_value_replayed_ops=+920`、`resource_summary_value_recomputed_ops=+10` である。
native release の RPN stage-only 測定では `resource_static_check=6915ms`、
`resource_initialized_moves=5998ms` だった。

これは base compile の固定費を少し削る checkpoint であり、0.5 秒未満の目標達成ではない。
引き続き、stdlib prechecked artifact、changed-function-only proof replay、typed expression
subtree query、codegen fragment cache を進める必要がある。

follow-up として、`SummaryWorklist` が共有 `ResourceSummaryDependencyGraph` の
`dependents` を clone せず `Cow` で借用できるようにした。legacy constructor は owned
dependents を保持するため、既存 test helper と独立構築経路は維持する。この変更も proof
key には影響しない。`tmp/rpn_borrowed_worklist_dependents_code_edit_20260531.json` では
base `compile_ms=9510` / `resource_static_check=8446.129ms`、unused local edit
`compile_ms=2251` / `resource_static_check=1943.803ms` だった。proof counter は前 checkpoint
と同じ形で、raw-init residual は `0` のままである。elapsed time は測定揺れの範囲であり、
この follow-up 単体を 0.5 秒目標の達成要因とは扱わない。

i32 scalar residual については、`ReprojectionValue` bypass を fact 種別へ分解した。
`tmp/rpn_i32_fact_kind_counters_final_code_edit_20260531.json` では、unused local 追加 edit が
`compile_ms=2219`、`resource_static_check=1922.104ms` で、edit delta の
`resource_summary_value_i32_scalar_return_facts_recomputed_ops=+10` は alias `+5` / offset `+5`
だけだった。reason 内訳は scalar type `+8` / parameter projection `+2`、condition 系は `0`
である。したがって、次に直すべき i32 residual は condition propagation ではなく、
alias / offset の stable mirror surface である。ただし base compile は同測定でも
`compile_ms=8931`、`resource_static_check=8318.313ms` であり、stdlib prechecked artifact と
Resource proof template の設計は引き続き優先する。

i32 scalar residual の完了 checkpoint では、debug-only ログで残った alias / offset を
`load<Option<i32>>` 型の leading raw `Deref`、`str` 全体の raw address carrier、
`str` からの `StorageOffset(Known(4))` carrier、symbolic offset 内 place の open generic
rebase に分けた。`StorageOffset` 自体を authority にはせず、raw address carrier ではない
型の offset は従来通り拒否する。修正後の `tmp/rpn_i32_reprojection_fix_measure_20260601.json` では、
same-session unused local edit の i32 scalar delta が
`resource_summary_value_i32_scalar_return_facts_recomputed_ops=0`、
`resource_summary_value_i32_scalar_return_facts_bypasses=0`、
`resource_summary_value_i32_scalar_return_facts_reprojection_value_bypasses=0` になった。
このため i32 scalar stable mirror の残差 issue は解決済みとする。ただし同測定の
base `compile_ms=9583`、edit `compile_ms=2292` はまだ性能目標に届いていない。
次の支配項は i32 scalar mirror ではなく、未変更 proof replay の固定費、stdlib prechecked
artifact、typed expression subtree query、codegen fragment cache の設計で削る。

当面の実装順は次の通りにする。

### 2026-06-01 owner obligation pass-cache lazy summary checkpoint

owner obligation pass-only cache により、RPN same-session edit では checker 本体の実行を
`0` へ落とせた。しかし、その前段の `compute_owner_return_summaries` が全関数固定点を
毎回構築していたため、`resource_static_owner_obligations` に秒単位の固定費が残っていた。

今回の checkpoint では、owner obligation pass entry が全関数で hit した compile に限り、
owner return summary の構築そのものを遅延して省略する。これは `OwnerReturnSummary` を
session-local な `TypeId` / projection ごと保存する cache ではない。すでに body hash、
dependency closure hash、source capability policy hash、type boundary hash で検証された
diagnostic-free pass entry が全関数に揃っている場合だけ、現在の owner obligation gate が
消費しない summary を作らないという整理である。

1 関数でも pass replay が miss した場合は、従来どおり owner return summary 固定点を構築し、
miss した関数だけ checker を実行する。したがって、この checkpoint は all-hit warm edit の
固定費削減であり、partial miss での owner summary reuse は解決しない。最終形は
`.neplproof` に進められる stable mirror として `OwnerReturnSummaryEntryV1` を設計し、
すべての `TypeId` / `PlaceProjection` / `OwnerProjectionSource` / extent / variant condition を
現在 compile の `TypeCtx` と function signature へ fail-closed に再投影できる場合だけ replay する。

測定では `tmp/rpn_owner_summary_lazy_skip_code_literal_20260601.json` の RPN 実コード文字列
literal edit が base `compile_ms=10254`、edit `compile_ms=2110` だった。edit の
`resource_static_owner_obligations` は `745.034ms` で、owner obligation pass cache checkpoint の
`1534.075ms` から下がった。edit delta は
`resource_owner_return_summary_recomputations=0`、
`resource_owner_return_summary_count=0`、
`resource_owner_return_summary_pass_cache_skip_functions=288`、
`resource_owner_obligation_function_checks=0`、
`resource_summary_value_owner_obligation_check_replay_hit_functions=288` である。

### 2026-06-01 Resource summary changed-function replay plan checkpoint

final initialized check と同じ changed-function plan を、raw alias / i32 scalar / raw-init
summary の preseed loop にも広げた。ただし summary fixed-point は caller が callee summary
index を読むため、final check のように「何も materialize せず pass を返す」ことはしない。
前回 compile で保存できた stable summary entry の key だけを snapshot に持ち、現在 compile
で affected ではない関数については dependency closure hash の再構築と replay probe を省き、
その key から現在の `TypeCtx` / function signature へ summary を再投影して index に入れる。

snapshot は関数 local fingerprint と stable summary key だけを持つ。`TypeId`、`Span`、
`SourceMap`、summary 本体は保持しない。関数 order、namespace、source capability policy、
body hash、type boundary、generic boundary の不整合、または stable key / stable value の
再投影失敗は通常 replay / conservative-all に戻す。subagent review で指摘された span 由来
symbol の過剰 invalidation については、`ResourceCallTarget::User`、`EffectOp::UserCall`、
`FunctionValueIdentity` の body hash 入力を `__def{file}_{start}_{end}` 正規化済み symbol
に揃えた。

測定では `tmp/rpn_summary_replay_plan_code_literal_20260601.json` の RPN 実コード文字列
literal edit が base `compile_ms=9593`、edit `compile_ms=1521` だった。edit stage は
`resource_static_check=1222.739ms`、`resource_static_initialized_moves=301.490ms`、
`resource_static_owner_obligations=805.523ms`、`resource_typecheck=163.275ms` である。
edit delta は `raw_alias_replay_probe_functions=0`、
`raw_alias_plan_skip_functions=288`、`i32_scalar_replay_probe_functions=0`、
`i32_scalar_plan_skip_functions=209`、`raw_init_replay_probe_functions=0`、
`raw_init_plan_skip_functions=151`、`initialized_function_check_plan_skip_functions=288` だった。

これは warm edit の summary preseed 固定費を削る checkpoint であり、まだ 0.1 秒以下の
式枝差し替え budget には届いていない。残りの edit 支配項は owner obligation lazy path の
残コスト、typecheck、lowering / effect / borrow の固定費である。base compile も
`compile_ms=9593` と秒単位なので、stdlib prechecked artifact と Resource proof template は
引き続き優先する。

- edit path: i32 scalar residual 解消後も残る秒単位の replay / fixed-cost を changed-function-only proof replay と typed expression subtree query へ分ける。
- base path: stdlib prechecked artifact と Resource proof template を優先し、初回 compile の fixed-point 探索空間を減らす。
- shared path: typed expression subtree query と codegen fragment cache は、warm edit と base artifact の両方から使える query 境界として設計する。

### 2026-06-01 `.neplmeta` public interface artifact checkpoint

`.neplmeta` の永続 codec へ進む前に、core compiler が返す in-memory public interface
artifact を追加した。これは base compile を下げるための依存 module 再 typecheck 削減に
向けた最初の typed metadata 境界である。

`.neplmeta` の payload は `TypedPublicSignatureTable` に限定する。ここには public callable、
public struct / enum / trait、impl header の stable text と deterministic hash だけを入れる。
`TypeId`、`Span`、`FileId`、`SourceMap`、typed HIR、Resource IR、diagnostic span は保存しない。
これらは compile session の arena や source-map allocation に結び付くため、cross-session
artifact に入れると stale hit の原因になる。

header には schema、compiler identity、target/profile、stdlib content hash、dependency public
surface hash、typed public signature hash、public entry count、source capability policy set hash、
private effect policy hash を入れる。private effect policy hash は常に `Some` とし、`memo_call`
や `PrivateCache` の mask rule が変わったときに古い interface artifact を受け入れない。

Web `CompilerSession` は `.neplmeta` artifact を slot と compiled-output cache entry に保持する。
compiled-output cache hit では cache entry の artifact を slot へ復元するだけで、Web 側で header
を再構築しない。stats JSON は `nepl_meta_artifact_present`、public entry 数、typed public
signature hash、payload consistency だけを公開し、payload 本体は出さない。

この checkpoint は「artifact を作って観測する」段階であり、まだ `.neplmeta` から typecheck
environment を組まない。次の根本対応は、import / prelude boundary で `.neplmeta` から public
callable/type/trait/impl surface を注入し、stdlib body の再 parse / 再 typecheck を減らすことである。

subagent review では、現在の `.neplmeta` payload は `TypedPublicSignatureTable` の stable
text / hash に限定されているため、そのままでは `Env` や `TypeCtx` を復元できないと確認した。
次に進める単位は body skip ではなく、per-module structured public surface を `.neplmeta`
payload に追加することである。保存する値は `TypeId`、`Span`、`SourceMap`、typed HIR ではなく、
module canonical path、public callable surface、public type constructor surface、trait bounds、
impl header、`noshadow` / symbol policy などの arena 非依存情報に限定する。

その後、typecheck materializer が structured public surface を現在 compile の fresh `TypeCtx`
と `Env` へ投影する。diagnostic span は artifact 内に保存せず import directive 側へ寄せる。
import / prelude boundary でこの materializer が使えるようになるまで、`.neplmeta` は base compile
time を大きく下げる authority ではなく、後続 artifact の安全な invalidation envelope として扱う。

### 2026-06-01 `.neplmeta` structured public surface checkpoint

`.neplmeta` payload に `TypedPublicSurfaceTable` を追加した。既存の
`TypedPublicSignatureTable` は stable text / hash の互換用 surface として残し、structured
surface は public callable、struct、enum、trait、impl header を enum/struct payload として
保持する。header には `structured_public_surface_hash` と
`structured_public_surface_entry_count` を追加し、payload consistency check でも typed text
signature と structured surface の両方を確認する。

この checkpoint でも `TypeId`、`Span`、`FileId`、`SourceMap`、`ImportResolution`、typed HIR、
Resource IR、diagnostic span は `.neplmeta` に保存しない。artifact 形状が変わったため
`.neplmeta` schema / artifact hash / compiler identity は v2 に上げた。

subagent review では、次の materializer 実装前に `Named(String)` だけの名義型参照、
名前・capability だけの generic parameter 参照、span-derived callable symbol を
authority にしてはいけないと確認した。現 structured surface は in-memory payload と
hash 境界を作る段階に留め、dependency body skip へ使う前に stable nominal identity、
binder-indexed generic parameter reference、stable public ABI symbol / field accessor surface を
追加する。

### 2026-06-01 `.neplmeta` public surface module split checkpoint

`TypedPublicSurfaceTable` の model / hash / builder / tests を
`nepl-core/src/typecheck/public_surface.rs` に分離した。`public_signature.rs` は
`TypedPublicSignatureTable` の stable text / hash 境界に戻し、`.neplmeta` materializer
向けの structured payload を持たない。

この分割は base compile 改善そのものではなく、次段階の materializer が stable nominal
identity、binder-indexed generic parameter reference、field accessor surface、stable public
ABI / link symbol を足しても、text signature cache 境界と構造化 payload 境界が混ざらないように
するための前提である。`typecheck.rs` の public re-export は維持し、外部 crate が参照する
`crate::typecheck::{PublicTypeTerm, TypedPublicSurfaceTable}` などの API は変えない。

### 2026-06-01 `.neplmeta` stable nominal surface checkpoint

structured public surface に `PublicNominalTypeIdentity` を追加した。これは `TypeCtx` の
`NominalStableTypeIdentity` から kind、source path、name、arity、definition hash を取り出し、
`TypeId` や `Span` を保存せずに public nominal type を compile session 間で対応付けるための
payload である。

public struct / enum surface は自身の identity を保持する。`PublicTypeTerm::Named` も、SourceMap
がある compile では参照先の stable nominal identity を保持する。SourceMap がない compile では
identity は `None` であり、materializer ではこれを authority として使わず fail-closed に拒否する。

この payload 形状変更に合わせ、structured surface hash namespace は
`neplg2-typed-public-surface-v2`、`.neplmeta` schema / artifact hash / compiler identity は v3
へ上げた。これにより、stable nominal identity を持たない古い `.neplmeta` artifact と同じ
contract として扱わない。

まだ trait identity、binder-indexed generic parameter reference、field accessor surface、
stable public ABI / link symbol、generic impl bound は残る。したがって、この checkpoint 後も
dependency body skip へ進む前に materializer authority の残件を解消する必要がある。

### 2026-06-01 `.neplmeta` binder-indexed generic surface checkpoint

structured public surface の generic parameter 表現を、binder metadata と binder reference に
分離した。`PublicTypeParam` は binder 側の parameter name / capability / index を表し、
type term 内の参照は `PublicTypeParamRef { binder_depth, index }` だけを保持する。これにより、
同名 `.T` が複数の binder に現れても、`.neplmeta` materializer は名前ではなく binder 位置で
現在 compile の fresh `TypeId` へ対応付けられる。

`binder_depth=0` は最も内側の binder を指す。generic function type が type parameter を導入する
場合、その function type の内側では新しい binder が depth 0 になり、外側 binder は depth 1
以降へ押し出される。type parameter を導入しない function type では新しい binder を作らないため、
外側 generic parameter の depth は変えない。

callable の trait bound は function type term の外側にある surface field だが、対象 parameter は
root function binder の `PublicTypeParamRef` で表す。binder が確定できない bound target は
`PublicTypeParamBoundTarget::Unbound`、term 側では `PublicTypeTerm::UnboundGenericParam` として
残す。これは推測して materialize する fallback ではなく、materializer が fail-closed に拒否して
通常の source load / typecheck へ戻るための明示的な不完全 payload である。

この payload 形状変更に合わせ、structured surface hash namespace は
`neplg2-typed-public-surface-v3`、`.neplmeta` schema / artifact hash / compiler identity は v4
へ上げた。binder-indexed generic reference を持たない古い `.neplmeta` artifact を同じ contract
として扱わない。

まだ `PublicTypeTerm::Named { identity: None }`、`PublicTypeTerm::UnboundGenericParam`、
`PublicTypeParamBoundTarget::Unbound` を materializer 本体の body-skip 判定へ接続する実装が必要である。

### 2026-06-01 `.neplmeta` materializer preflight checkpoint

`TypedPublicSurfaceTable` に materializer preflight を追加した。これは `.neplmeta`
materializer 本体ではなく、structured public surface を現在 compile の `TypeCtx` / `Env`
へ投影する前に、body skip してはいけない surface を fail-closed に列挙する境界である。

preflight は primitive だけで構成された callable surface を通す。一方で、name-only
nominal type、対応 binder を持たない generic parameter、対応 binder を持たない trait
bound target、stable trait identity を持たない trait reference、identity を持たない
public struct / enum surface は blocker として返す。

これにより、`.neplmeta` materializer の実装が進んだ段階でも、`Named(String)` や
`UnboundGenericParam` を推測で current session の型へ対応付ける経路を作らない。今後は
trait stable identity、field accessor surface、stable public ABI/link symbol、generic impl
bound、reexport / prelude edge と module canonical path を追加し、preflight blocker を
減らしていく。
field accessor surface、stable public ABI / link symbol、generic impl bound、reexport /
prelude edge と module canonical path も引き続き dependency body skip の前提として残る。

### 2026-06-01 `.neplmeta` stable trait surface checkpoint

structured public surface に `PublicTraitIdentity` を追加した。trait surface 自体と、
callable bound / trait impl header に現れる `PublicTraitRef` は、SourceMap がある場合に
source path、trait name、arity、definition hash を保持する。

definition hash は trait type parameter、capability、method name、method type surface から
作る。doc comment、method body、Span、TypeId、typed HIR、Resource IR は含めない。これにより
trait method body や doc だけの編集では `.neplmeta` surface を捨てず、method signature、
capability、source path、trait name、arity が変わった場合は stale artifact として扱える。

`PublicTraitRef` は identity がある場合、materializer preflight の
`MissingTraitIdentity` blocker にならない。SourceMap がない compile では identity は `None`
のままで、従来どおり fail-closed に body skip を止める。

この payload 形状変更に合わせ、structured surface hash namespace は
`neplg2-typed-public-surface-v4`、`.neplmeta` schema / artifact hash / compiler identity は v5
へ上げた。trait identity を持たない古い `.neplmeta` artifact を同じ contract として扱わない。

この checkpoint でも `.neplmeta` materializer 本体はまだ未実装である。次は field accessor
surface と stable public ABI / link symbol、generic impl parameter / bound、reexport /
prelude edge、module canonical path を揃えてから、fail-closed materializer を import /
prelude boundary へ接続する。

### 2026-06-01 `.neplmeta` callable authority checkpoint

`PublicCallableSurface` に `PublicFieldAccessorKind` と `PublicCallableLinkSymbol` を追加した。
field accessor helper は同じ型・effect・arity の普通の関数とは Resource / SourceCapability
上の意味が異なるため、`get_field` / `get_field_ref` / `set_field` 由来の helper kind を
structured surface に保存する。

link symbol は span-derived `mangle_function_symbol_for_def` を artifact authority にしない。
SourceMap がある callable について、source path、public callable name、structured type
surface の signature hash から stable link symbol payload を作る。body-only edit では
変わらず、signature edit や source path 変更では変わるため、cross-session materializer が
Span / FileId に依存せず ABI 境界を確認できる。

materializer preflight は stable link symbol を持たない callable を
`MissingCallableLinkSymbol` blocker として fail-closed に拒否する。SourceMap がない compile や、
まだ stable ABI を付けられない artifact は body skip へ進まない。

この payload 形状変更に合わせ、structured surface hash namespace は
`neplg2-typed-public-surface-v5`、`.neplmeta` schema / artifact hash / compiler identity は v6
へ上げた。callable authority を持たない古い `.neplmeta` artifact を同じ contract として扱わない。

この checkpoint でも `.neplmeta` materializer 本体はまだ未実装である。generic impl parameter /
bound、reexport / prelude edge、module canonical path を揃えてから、fail-closed materializer を
import / prelude boundary へ接続する。

### 2026-06-01 `.neplmeta` generic impl surface checkpoint

`ImplInfo` に impl header の generic binder と bound environment を保持し、`PublicImplSurface`
へ `type_params` と `type_param_bounds` として投影するようにした。

従来の impl surface は `public_impl_surface` が空の generic map で target type と trait
application を変換していた。そのため `impl<.T: Touch> Touch for Holder .T` の `.T` が
`PublicTypeParamRef` にならず、`.neplmeta` materializer が名前から推測するか fail-closed に
止まるしかなかった。

新しい surface では、impl target、trait application args、impl bound target が impl 自身の
binder を参照する。bound trait には `PublicTraitIdentity` を保持するため、public trait の
bound は stable identity で復元でき、private trait bound や identity 欠落は preflight blocker
として扱える。

`PublicImplKind::Trait` は public impl header に必要な trait application だけを持つ。
trait definition 内部の `Self` type term は impl header の外部 authority ではなく、fresh
session で直接 materialize する対象ではないため structured impl surface から外した。

この payload 形状変更に合わせ、typed public signature hash namespace は
`neplg2-typed-public-signature-v2`、structured public surface hash namespace は
`neplg2-typed-public-surface-v6`、`.neplmeta` schema / artifact hash / compiler identity は v7
へ上げた。generic impl binder / bound を持たない古い `.neplmeta` artifact を同じ contract
として扱わない。

この checkpoint でも `.neplmeta` materializer 本体はまだ未実装である。次は reexport /
prelude edge と module canonical path を per-module surface に加えたうえで、fail-closed
materializer を import / prelude boundary へ接続する。

### 2026-06-01 `.neplmeta` trait Self surface checkpoint

trait method signature 内の `Self` を `PublicTypeTerm::TraitSelf` として structured public
surface へ保存するようにした。

trait collection は `Self` を fresh type variable として作る。一方で従来の
`public_trait_surface` は trait type parameter だけを binder map に入れていたため、
`pub trait Show: fn show %fn Self i32 ...` の `Self` が `UnboundGenericParam` になっていた。
これでは stable trait identity があっても materializer preflight で generic blocker が残る。

`TraitSelf` は trait method surface 内だけで許可される implicit receiver type であり、通常の
`.T` generic binder とは別に扱う。trait method 以外の public surface に現れた場合は
`TraitSelfOutsideTraitMethod` blocker として fail-closed に止める。

この payload 形状変更に合わせ、structured public surface hash namespace は
`neplg2-typed-public-surface-v7`、`.neplmeta` schema / artifact hash / compiler identity は v8
へ上げた。trait-local `Self` を unbound generic として保存していた古い artifact を同じ
contract として扱わない。

この checkpoint でも `.neplmeta` materializer 本体はまだ未実装である。次は reexport /
prelude edge と module canonical path を per-module surface に加えたうえで、fail-closed
materializer を import / prelude boundary へ接続する。

### 2026-06-01 `.neplmeta` module edge surface checkpoint

`.neplmeta` payload に module edge surface を追加した。これは loader が既に持っている
root module の import / prelude / include surface を、artifact materializer が使える
arena 非依存 payload として保存するための checkpoint である。

module edge surface は canonical module path、default prelude path、`#no_prelude`、
implicit default prelude の有無、direct dependency edge を持つ。edge は
`Prelude` / `Import` / `Include`、resolved canonical target path、visibility、import clause、
public re-export eligible flag、source order を保持する。`PathBuf`、`FileId`、`Span`、
`ImportResolution`、typed HIR、Resource IR は保存しない。

`.neplmeta` header には `module_surface_hash` と `module_dependency_edge_count` を追加した。
payload consistency check も module surface hash / edge count を検証するため、artifact decode 後に
header と payload がずれた module edge authority を受け入れない。

この payload 形状変更に合わせ、`.neplmeta` schema / artifact hash / compiler identity は v9
へ上げた。Web `CompilerSession` は loader の `LoadResult` から module surface を受け取り、
compile result の `.neplmeta` artifact と stats JSON へ反映する。

この checkpoint は export / re-export table の完全 materializer ではない。次は re-export された
public entry の origin module / origin name / exported name / kind を module identity ベースで保存し、
target artifact missing や dependency mismatch を enum reason で fail-closed に拒否する。

### 2026-06-01 `.neplmeta` export surface checkpoint

`.neplmeta` payload に public export surface と re-export projection surface を追加した。

これまでは structured public surface が callable / struct / enum / trait / impl header を保持し、
module edge surface が import / prelude / include edge を保持していた。しかし local public name と
`pub #import` 由来の公開投影を結ぶ surface がなかったため、materializer は依存 module の body を
読まずに export authority を復元できなかった。

新しい export surface は local export と re-export projection を分けて保持する。local export は
exported name、origin module path、origin name、kind (`Callable` / `Struct` / `Enum` / `Trait`) を
持つ。`Impl` は名前 export ではなく trait lookup authority なので local export には入れず、
structured public surface の impl header と visibility map で扱う。

re-export projection は source order、target module path、edge kind、import clause を保持する。
`Open`、selective import、glob、merge をこの段階で推測展開せず、target `.neplmeta` artifact を
読めた materializer だけが fail-closed に展開する。`PathBuf`、`FileId`、`Span`、
`ImportResolution`、typed HIR、Resource IR は保存しない。

`.neplmeta` header には `export_surface_hash`、`local_export_count`、
`reexport_projection_count` を追加し、payload consistency check でも export surface hash / count の
不一致を拒否する。payload 形状変更に合わせて schema / artifact hash / compiler identity は v10 に
上げた。Web `CompilerSession` stats JSON には local export count、re-export projection count、
export surface hash を追加した。

次 checkpoint は `.neplmeta` materializer MVP である。stdlib public surface のうち local export、
`ImportClause::Open`、simple named import を fail-closed に materialize し、未対応の include、
merge、alias collision、glob ambiguity、impl lookup は通常 load / typecheck へ戻す。

### 2026-06-01 `.neplmeta` materializer MVP gate checkpoint

`.neplmeta` artifact に materializer MVP gate を追加した。

この checkpoint はまだ dependency body skip や `TypeCtx` / `Env` 注入を行わない。目的は、
body skip の前段で artifact 全体が stdlib public surface materializer へ渡せるかを
fail-closed な enum reason で判定し、Web / Node の stats JSON から観測できるようにすることである。

`NeplMetaArtifact::materializer_mvp_reject` は、payload consistency、module surface、export
surface、module identity、public surface preflight blocker、unsupported import projection を
一か所で確認する。`Impl` は名前 export ではなく trait lookup authority なので、この MVP gate では
`UnsupportedImplLookup` として通常 typecheck へ戻す。

MVP gate が受け入れる範囲は、local export と `ImportClause::Open`、alias なし selective import である。
`include` は現行 loader では AST inline 境界なので import と同一視しない。`merge`、default alias、
alias、glob は target artifact と collision / ambiguity 判定なしに展開できないため、専用 reason で
拒否する。

Web `CompilerSession` stats JSON には `nepl_meta_artifact_materializer_mvp_ready` と
`nepl_meta_artifact_materializer_mvp_reject_code` を追加した。現段階では ready が true でも
「body skip してよい」という意味ではなく、次 checkpoint の `typecheck/materializer` で
fresh `TypeCtx` / `Env` へ投影する前提条件が揃ったことだけを表す。

次 checkpoint では、artifact-level gate と実際の typecheck materializer を分けたまま、
`PublicTypeTerm -> TypeId` の復元、local public callable の `Env` 注入、Open / simple named import
の projection を小さく実装する。

### 2026-06-01 `.neplmeta` typecheck materializer callable MVP checkpoint

`nepl-core/src/typecheck/materializer.rs` を追加し、`.neplmeta` の structured public surface を
現在 compile session の fresh `TypeCtx` / `Env` へ投影する最初の内部 API を実装した。

この checkpoint は stdlib dependency body skip そのものではない。目的は、artifact-level gate を
通過した public callable surface について、依存側の prefix call 解決が読む callable 候補を
`TypeId` と `Binding` として安全に再構築できるかを固定することである。`TypeId`、`Span`、
`SourceMap`、typed HIR、Resource IR proof は `.neplmeta` から復元しない。

受け入れる型:

- `unit` / `i32` / `u8` / `f32` / `bool` / `char` / `str` / `never`。
- tuple。
- function type。`PublicTypeParamRef { binder_depth, index }` は binder stack で解決する。
- box / reference type。これは型境界の復元だけであり、owner / borrow proof の再利用ではない。

拒否する surface:

- callable 以外の struct / enum / trait / impl surface。
- stable link symbol を持たない callable。
- link symbol name と entry name が一致しない callable。
- field accessor callable。
- trait bound を持つ generic callable。
- name-only または stable identity 付きであっても、まだ名義型 materializer がない named type。
- `TraitSelf`、unbound generic parameter、type application。
- 既存 value との name conflict、および `no_shadow` 同signature conflict。

subagent review 後、materializer は two-phase staging にした。table 内の後続 entry で拒否が
起きた場合、前半で staging した binding は `Env` へ入れない。これにより、artifact materialize
失敗後に通常 source load / typecheck fallback へ戻っても、半端な imported callable 候補が
残らない。

また callable surface の内部整合性も materialize 前に確認する。entry kind が callable であること、
`surface.ty` が function type であること、`arity` と parameter 数、surface effect と function
type effect、`PublicCallableLinkSymbol.signature_hash` と public type term stable hash が一致する
ことを検査する。壊れた `.neplmeta` payload が self-consistent な outer hash を持っていても、
この局所 boundary で fail-closed に止める。

materialized callable symbol は `PublicCallableLinkSymbol` の source path / name / signature hash から
決定的に作る。span-derived mangle は使わないため、同じ dependency artifact を別 compile
session で読んでも同じ callable symbol になる。一方で `def_id` は `None` のままであるため、
`@func` / `memo_call @func` のような function value identity 依存の経路は、stable function
identity materializer が入るまで通常 source typecheck fallback 側に残す。

この実装により、`.neplmeta` は typed public surface を「保存できる」だけでなく、
primitive / generic callable に限って typecheck candidate として復元できる段階へ進んだ。
次の根本対応は、import / prelude boundary で target artifact を引いて local export と
Open / simple named import projection をこの materializer へ渡すこと、および named type /
trait bound / field accessor / function value identity を専用 authority で順に追加することである。

### 2026-06-01 `.neplmeta` import projection checkpoint

target `.neplmeta` artifact を読めた後に、import clause で見える public surface だけを
typecheck materializer へ渡す projection API を追加した。

この checkpoint もまだ dependency body skip ではない。現時点の `CompilerSession` は直近 compile
artifact slot だけを持ち、module path keyed な依存 artifact map や disk / IndexedDB codec を
持っていない。そのため、loader が import / prelude boundary で target artifact を引く処理へは
まだ接続しない。今回の目的は、target artifact が存在する場合でも「何を `Env` へ注入してよいか」
を artifact layer の純粋関数として固定することである。

現在の境界:

- `materializer_local_export_public_surface_mvp` は local export を kind 付きで投影する。
- `materializer_import_public_surface_mvp` は `Open` / clause なしでは local export 全体、
  alias なし selective import では指定名の export だけを投影する。
- alias、glob、merge、default alias は visible name 書き換えや衝突判定が必要なので拒否する。
- re-export projection はさらに target artifact を読む必要があるため、この checkpoint では
  `UnsupportedReexportProjection` として拒否する。
- `Struct` / `Enum` / `Trait` export は projection で保持する。ただし current session への
  登録は named type / trait materializer が別に fail-closed に判定する。
- missing selective name は「存在しない」と推測せず、re-export 未展開の可能性も含めて専用 reason で
  fail-closed にする。

この projection により、前 checkpoint の `typecheck/materializer` は、artifact 全体ではなく
import clause で見える export subset だけを受け取れるようになった。次の段階では、module path
keyed な `.neplmeta` artifact store を `CompilerSession` / loader cache に持たせ、target artifact
header compatibility と dependency public surface hash を確認したうえで、この projection と
typecheck materializer を import / prelude boundary に接続する。

### 2026-06-01 LLVM dual CI shard checkpoint

GitHub Actions run `26728316260` では、`llvm-dual-test` の `tests` / `stdlib`
2 shard がどちらも 18 分の外側 timeout に達し、`exit code 137` で終了した。
これは LLVM backend の個別失敗ではなく、full dual backend verification の workload が
CI job boundary に対して大きすぎ、途中結果を残しても job 自体が完了できない構造である。

`nodesrc/tests.js` に `--shard INDEX/TOTAL` を追加した。shard は、入力から集めた元の
doctest case 集合を stable sort し、case identity を SHA-256 hash してから分割する。
その後で wasm / LLVM の runnable case を派生するため、`--runner all --llvm-all
--strict-dual` でも同じ original case が同じ shard に残り、wasm / LLVM の対応関係が
shard によって分断されない。

`llvm-dual-test` は `tests` 4 shard と `stdlib` 4 shard に分割した。`tests` の tree
suite は shard 1 だけで実行し、他の shard は `--no-tree` にする。これにより、tree suite
を全 shard で重複実行せず、full dual backend の主目的である doctest backend 比較に CI 時間を
使う。

各 shard は個別 JSON を upload し、Pages final bundle で `nodesrc/merge_doctest_json.js`
を使って `tests-dual-tests.json` / `tests-dual-stdlib.json` へ戻す。merge 後の JSON は
canonical な Pages input として扱えるが、`shards` field には shard ごとの summary と
scan metadata を残すため、timeout や partial report が混じった場合もどの shard 由来かを追跡できる。

これは compile performance 自体を改善するものではない。base compile の主要ボトルネックである
`resource_static_initialized_moves` / `resource_static_check` は別途、`.neplmeta` と Resource
summary cache / proof template の設計で削る必要がある。この shard は、現在の重い compiler を
CI 上で観測し続けるための workload boundary 修正である。

### 2026-06-01 `.neplmeta` source key invalidation checkpoint

`.neplmeta` header に `source_key_hash` を追加した。これは typed public signature や
structured public surface の hash とは別に、保存済み artifact が現在の module source に
由来するかを typecheck 前に確認するための境界である。

`source_key_hash` は既存の `compiled_source_cache_key_part` から作る。この source key は
lexer token stream 由来であり、通常コメント、doc comment、span だけの変更は無視する。一方で、
literal、identifier、directive、indent / dedent、raw wasm / llvm text など、compile 結果に
影響し得る token は保持する。したがって、dependency body skip や import materializer へ進む時に、
public signature が同じまま式 body だけが変わった artifact を誤って受け入れる経路を閉じられる。

body skip の前段では、typed public signature や structured public surface をまだ作れない。
そのため `NeplMetaArtifactPreTypecheckEnvelope` を別に追加し、target/profile、stdlib content、
dependency public surface、module surface、source capability policy、private effect policy、
source key だけで payload decode 前の照合を行えるようにした。typed public signature と
structured public surface は、この pre-typecheck envelope を通過した後の payload consistency
と materializer の責務として残す。

`SourceMap` または `.neplmeta` の canonical module path がない場合、`source_key_hash` は
`None` になる。これは「安全に再利用できる」という意味ではなく、body skip の前提を証明できない
artifact として通常 load / typecheck へ戻すための fail-closed 値である。`NeplMetaArtifactStore`
と materializer MVP は `source_key_hash=None` の artifact を拒否する。将来 disk / IndexedDB
codec を追加する場合も、payload decode や materializer 実行より前に pre-typecheck envelope と
header の `source_key_hash` を照合する。

Web `CompilerSession` の stats JSON には `nepl_meta_artifact_source_key_hash` を追加した。
値が 0 の場合は artifact がない、または source key を作れないことを表す観測値であり、
artifact payload 本体や source text は公開しない。

### 2026-06-01 `.neplmeta` pre-typecheck store projection checkpoint

`NeplMetaArtifactStore` に `materializer_import_public_surface_pre_typecheck_mvp` を追加した。
この API は body skip そのものではなく、loader/import boundary で target module の source と
module edge surface が分かった段階で、保存済み `.neplmeta` artifact から materializer 入力を
取り出せるかを probe するための境界である。

従来の store projection は full `NeplMetaArtifactHeader` を expected value として要求していた。
しかし full header には typed public signature hash と structured public surface hash が含まれ、
それらは target module body の typecheck 後にしか得られない。新しい API は
`NeplMetaArtifactPreTypecheckEnvelope` を受け取り、payload decode 前に確認できる
schema、compiler identity、target/profile、stdlib content hash、source key、dependency public
surface hash、module surface hash、source capability policy、private effect policy だけを照合する。

pre-typecheck envelope が通った後も、store は payload consistency と MVP projection を続けて
確認する。成功時に返るのは `TypedPublicSurfaceTable` だけであり、`TypeCtx` / `Env` への注入や
依存 module AST inline の省略はまだ行わない。失敗時は `MissingArtifact`、`PayloadConsistency`、
`Compatibility(SourceKey / DependencyPublicSurface / ModuleSurface / ...)`、
`Projection(...)` の enum reason で通常 load / typecheck fallback へ戻す。

この checkpoint により、次の loader/import probe は「artifact がない」「source が古い」
「dependency surface が違う」「import clause が MVP 範囲外」のどれで fallback したかを
文字列解析なしに区別できる。body skip を有効化する前に、この probe の hit/reject 統計を
Web `CompilerSession` から観測できるようにする。

### 2026-06-01 `.neplmeta` pre-typecheck probe observation checkpoint

`NeplMetaArtifactStoreStats` に pre-typecheck probe 専用の統計を追加した。既存の
`hits` は module path keyed store 内に artifact が存在した回数であり、projection 成功や
安全な body skip を意味しない。そのため、probe の `attempts`、`projected`、missing artifact、
payload reject、compatibility reject、projection reject、projected entry count を別 field として
記録する。

last reason は `last_pre_typecheck_probe_reject_kind` と
`last_pre_typecheck_probe_reject_code` に分けた。kind は missing / payload / compatibility /
projection の大分類であり、code はそれぞれの enum reason を安定した数値に畳んだ値である。
これにより Web playground や benchmark で、`SourceKey` mismatch と `UnsupportedAlias` のように
次に直す場所が違う fallback reason を文字列解析なしに分けられる。

この checkpoint でも compile path の意味は変えない。通常 compile は pre-typecheck probe を
まだ呼ばないため、Web `loader_cache_stats_json` では probe field が 0 として存在することだけを
確認する。次の loader/import 接続では、この統計を先に見ながら fallback rate を測り、body skip
へ進んでよい edge を限定する。

### 2026-06-01 `.neplmeta` session root pre-typecheck probe checkpoint

Web `CompilerSession` の実 compile path に、保存済み root `.neplmeta` artifact を
pre-typecheck envelope で照合する観測 probe を接続した。これは dependency body skip ではなく、
現在の loader/source-map 境界で artifact が再投影可能かを測るための安全な staging である。

接続条件:

- `.neplmeta` store が空でない場合だけ probe する。初回 compile は missing artifact として数えず、
  実際に再利用候補が存在する二回目以降だけを測る。
- stdlib overlay compile では従来通り loader/resource/proof cache を bypass し、probe store も
  compile path へ渡さない。
- probe は `materializer_import_public_surface_pre_typecheck_mvp` の戻り値を使わず、統計だけを
  更新する。`TypeCtx` / `Env` 注入、依存 module AST inline 省略、Resource IR skip は行わない。

固定した観測:

- import dependency の body-only edit では、root source key と dependency public surface が変わらない
  場合でも、現 payload の materializer blocker が残る artifact は projection reject として数えられる。
  store hit を body-skip ready と誤読しないための確認である。
- root source の literal edit では、typed public signature が変わらなくても source key mismatch で
  compatibility reject になり、projection 判定へ進まない。

subagent review では、依存 module 単位の本命 probe は `Loader::process_directives_with` の
prelude/import `load_file_with` 成功直後に置き、`Include` や AST merge を変えず観測専用 hook として
接続するのが安全と確認した。この checkpoint はその前段であり、次は import/prelude edge ごとの
module surface と import clause を渡す loader hook へ進む。

### 2026-06-01 `.neplmeta` import/prelude edge pre-typecheck probe checkpoint

Loader が実際に load した stdlib `#prelude` / `#import` edge から、
`.neplmeta` pre-typecheck envelope 用の観測値を作るようにした。root artifact probe と同じ
`NeplMetaArtifactStore` compatibility / projection 判定を使うが、統計は root probe と分けて
`pre_typecheck_edge_probe_*` に記録する。

edge envelope の source identity は root compile の `SourceMap` から作らない。loader は target
module の source key と、target file 単体の source capability policy set hash を probe に含める。
root file や同時に load された別 dependency の capability policy を混ぜると、同じ stdlib artifact が
呼び出し元 root によって compatibility reject されるためである。dependency public surface hash は
別 field として保持し、target の import/prelude 公開面変更はそちらで invalidation する。

安全境界:

- edge probe は `NeplMetaArtifactStore` が空でない compile だけで収集する。初回 compile や
  artifact 再投影候補がない compile では、dependency edge ごとの source 再読込や dependency hash
  計算を走らせず、base compile の固定費を増やさない。
- `#include` は AST merge 境界であり dependency artifact 境界ではないため、probe 対象にしない。
- non-stdlib VFS edge は `.neplmeta` store lookup 対象にしない。stdlib overlay compile でも
  従来通り loader / resource / proof cache と `.neplmeta` store を bypass する。
- probe は統計だけを更新し、`TypedPublicSurfaceTable` を `TypeCtx` / `Env` に注入しない。
  dependency AST inline 省略、Resource IR skip、codegen skip も行わない。

この checkpoint で分かるのは「import/prelude target artifact がまだ store にない」
「store にあるが envelope / projection で拒否された」「projection 可能だった」のどれかである。
body skip や stdlib prechecked artifact の実利用は、edge probe の fallback reason を十分に観測してから
次段の materializer 接続で行う。

固定した観測:

- root artifact store が埋まった後の dependency body-only edit では、実際に load された stdlib
  prelude/import edge だけが edge probe attempt として数えられ、現時点では target artifact が
  store にないため missing artifact として fallback する。
- `#no_prelude` + `#include` のみの compile では、store が埋まった二回目でも edge probe attempt は
  0 のままであり、include を dependency artifact edge と誤認しない。
- loader 単体 regression では、edge probe の source capability policy hash が root compile 全体の
  `SourceMap` hash ではなく target module file 単体の hash であることを固定した。

### 2026-06-01 `.neplmeta` stdlib dependency artifact producer checkpoint

Web `CompilerSession` に、import/prelude edge probe で見つかった bundled stdlib dependency の
`.neplmeta` artifact を生成して store へ入れる producer を追加した。これは typecheck body skip ではなく、
次回以降の edge probe が `MissingArtifact` で止まらず、compatibility / projection reason まで進めるための
中間 artifact 供給段階である。

producer は `compile_nepl_meta_artifact_with_source_identity` を使い、target module を typecheck した時点で
止まる。Resource IR static check、drop 挿入、wasm codegen は行わない。`.neplmeta` は依存側の公開 interface
authority であり、実行 body や Resource proof の authority ではないためである。source key と source
capability policy は root compile の `SourceMap` から再計算せず、loader の edge probe が target source 単位で
計算した値を header へ固定する。

stdlib dependency を root として読み直すと、`std/prelude_base` のような module に root 専用の既定 prelude
注入がかかり、自己循環 import を作る。そのため `Loader::load_dependency_inline_with_provider_and_cache` を追加し、
producer では edge target を non-root module として読み直す。これは元の root compile で import/prelude target が
読まれた条件に合わせるための loader 境界であり、通常の root compile path は変更しない。

安全境界:

- `.neplmeta` store が空の初回 compile では edge probe を収集しない。base compile の固定費を増やさないため、
  dependency artifact producer は二回目以降、edge probe 材料が既にある compile でだけ動く。
- stdlib overlay / `/stdlib` VFS override では producer も store lookup も従来通り bypass し、bundled stdlib
  artifact と overlay artifact を混ぜない。
- target path は bundled stdlib root 配下に限定し、non-stdlib import や `#include` は dependency artifact
  producer の対象にしない。
- store に同じ pre-typecheck envelope と互換な artifact が既にある場合は再 typecheck しない。この確認は
  `has_pre_typecheck_compatible_artifact` で行い、実 materializer probe の hit/miss 統計とは混ぜない。
- 生成した artifact は store へ入れるだけで、`TypedPublicSurfaceTable` を `TypeCtx` / `Env` へ注入しない。
  dependency AST inline 省略、Resource IR skip、codegen skip は次 checkpoint 以降の別作業である。

固定した観測:

- 1 回目 compile は root `.neplmeta` だけを store し、edge probe attempts は 0 のままである。
- 2 回目 compile は stdlib edge probe を実行し、probe 自体は既存どおり missing artifact として fallback する。
  compile 成功後に、その edge target の `.neplmeta` artifact を store へ追加する。
- 3 回目 compile は同じ edge probe で missing artifact を増やさず、現在の MVP materializer が対応していない
  public surface について projection reject まで進む。これにより、次に直すべき理由が「artifact 未生成」ではなく
  「materializer が未対応」に移る。

### 2026-06-01 `.neplmeta` projection blocker detail checkpoint

`.neplmeta` store stats に、projection reject が `PublicSurfaceBlocker` だった場合の
blocker reason code と entry kind code を追加した。これは性能改善そのものではなく、
次の materializer 実装単位を誤らないための観測 boundary である。

これまで Web stats は `rejectKind=Projection` / `rejectCode=6` までしか持たず、
`PublicSurfaceBlocker` の中身を失っていた。`MissingTraitIdentity`、`MissingNamedTypeIdentity`、
`MissingCallableLinkSymbol`、`UnboundGenericParam` などは必要な materializer authority が異なるため、
同じ code へ丸めると次の高速化作業が根拠を失う。

今回追加した値:

- `nepl_meta_artifact_materializer_mvp_public_surface_blocker_reason_code`
- `nepl_meta_artifact_materializer_mvp_public_surface_blocker_entry_kind_code`
- `nepl_meta_artifact_store_last_pre_typecheck_probe_projection_blocker_reason_code`
- `nepl_meta_artifact_store_last_pre_typecheck_probe_projection_blocker_entry_kind_code`
- `nepl_meta_artifact_store_last_pre_typecheck_edge_probe_projection_blocker_reason_code`
- `nepl_meta_artifact_store_last_pre_typecheck_edge_probe_projection_blocker_entry_kind_code`

`std/prelude_base` の dependency artifact は 3 回目 compile で `MissingArtifact` を増やさず、
`PublicSurfaceBlocker` まで進む。最初の blocker は
`MissingTraitIdentity` (`reason_code=7`) かつ `Impl` surface (`entry_kind_code=5`) である。
具体的には `std/prelude_base` が `core/traits/copy` を `@merge` import し、
そこから `copy/primitive` の `Clone` / `Copy` trait と primitive impl 群が入る。
これらの trait は public export API ではないが、prelude capability registration には必要な
semantic surface である。

次の根本対応は、callable-only materializer を拡張して `Clone` / `Copy` を例外的に通すことではない。
trait table、impl table、capability registration を `.neplmeta` から fail-closed に復元する
[`.neplmeta trait and impl materializer needed for prelude capability surface`](../../issues/items/ISS-20260601T193116311Z-NEPLMETA-TRAIT-IMPL-MATERIALIZER-NEEDED-D3A0C2F1.md)
を進める。`memo_call` / private cache purity / function value identity はこの surface
materializer の authority ではないため、`.neplmeta` 由来 callable は引き続き `def_id=None`
の direct call 専用に留める。

### 2026-06-01 `.neplmeta` semantic trait surface checkpoint

`.neplmeta` structured public surface は、export される public API と、dependency
typecheck に必要な semantic support surface を分けて保持するようになった。private
`Clone` / `Copy` trait は public export にはしないが、impl header や callable bound が
参照する場合は stable trait identity 付きで artifact に残す。

これにより `std/prelude_base` dependency artifact probe の最初の blocker は
`MissingTraitIdentity` (`reason_code=7`) から `MissingNamedTypeIdentity`
(`reason_code=3`) へ進んだ。entry kind は引き続き `Impl` (`entry_kind_code=5`) である。
つまり trait identity の欠落は解消し、次の root gap は `MemPtr .T` などの nominal type
application を current session の `TypeCtx` へ安全に materialize することである。

export surface は `TypedPublicSurfaceEntry.exported=true` の entry だけを local export に
する。semantic-only trait は import 先の visible namespace には出さず、後続の trait /
impl materializer が impl header を復元するための authority としてだけ扱う。

### 2026-06-01 `.neplmeta` backend scalar and local int128 capability checkpoint

`.neplmeta` materializer preflight は、compiler-defined backend scalar と user-defined
nominal type を分けて扱うようになった。`i64` / `u64` / `f64` / `u32` は
`BackendScalarType` が所有する named scalar domain なので、stable nominal identity がなくても
`MissingNamedTypeIdentity` にはしない。materializer 本体も同じ enum domain から `TypeCtx` の
backend scalar `TypeId` を復元する。文字列の再列挙ではなく、layout / wasm / LLVM / Resource
stable key と同じ authority を使うため、support set の drift を Rust 側の enum 境界へ集約できる。

一方で `i128` / `u128` は backend scalar ではない。これらは `core/math/int128/types` が
所有する public struct であり、`copy/primitive` に primitive capability impl を置くと、
default prelude の `.neplmeta` surface が型定義を持たないまま name-only nominal term を
持ってしまう。そこで `i128` / `u128` の `Clone` / `Copy` impl は型定義 module へ移し、
prelude の primitive capability module は本当に primitive な型だけを所有する形に戻した。

この checkpoint 時点では、`std/prelude_base` の dependency edge probe で一部の stored stdlib
artifact が projection success まで進み、残る projection reject は `PublicSurfaceBlocker` ではなく
`UnsupportedExportKind` (`reject_code=11`) になった。これは non-callable export / semantic trait・impl
surface を current session の registry へ復元する段階が次の root gap であることを示していた。
`UnsupportedExportKind` では blocker reason / entry kind code は 0 のままにし、古い
`MissingNamedTypeIdentity` 詳細を stats に残さない。

この変更は `i128` / `u128` を backend scalar として扱うものではない。同名 user nominal と
backend scalar の衝突は、今後 `.neplmeta` の type term に compiler-owned scalar tag を持たせるか、
backend scalar 名を予約型名として診断する設計でさらに固定する必要がある。

### 2026-06-01 `.neplmeta` non-callable export projection checkpoint

`.neplmeta` projection は、local export を callable だけに制限せず、artifact が保持する
export kind と同じ `TypedPublicSignatureKind` の structured public surface entry を返すようにした。
`Callable` / `Struct` / `Enum` / `Trait` は projection できる。`Impl` は visible export ではなく、
semantic-only support surface として後段の trait / impl materializer が扱う。

この checkpoint も body skip 完了ではない。projection は artifact payload を
`TypedPublicSurfaceTable` に戻す純粋な境界であり、current session の `TypeCtx` / `Env` へ
何を登録できるかは `typecheck/materializer` が別に判定する。現時点の materializer 本体は
callable 以外をまだ `UnsupportedSurfaceKind` として fail-closed に扱う。

tree regression では、dependency body-only edit の root pre-typecheck probe が
`UnsupportedExportKind` reject ではなく projection success まで進むようになった。また、
stored stdlib dependency artifact の edge probe でも複数 artifact が projection success し、
last reject code は 0 に戻る。残る root gap は、artifact MVP gate の `UnsupportedImplLookup`
および non-callable surface を semantic registry / visible namespace へ安全に materialize する処理である。

次の作業単位:

- `Struct` / `Enum` / `Trait` surface を stable nominal / trait identity から `TypeCtx` と trait table へ復元する。
- semantic-only `Impl` を visible export に混ぜず、validated impl registry と capability registration へ注入する。
- `PublicTypeTerm::Named { identity: Some(...) }` と `Apply` を実体化してから impl target / trait application を復元する。
- re-export projection は target artifact chain と衝突判定が必要なので、引き続き fail-closed に残す。

### 2026-06-01 `.neplmeta` nominal and trait definition materializer checkpoint

`typecheck/materializer` に、projection 済み `Struct` / `Enum` / `Trait` surface を current session の
semantic table へ staging する入口を追加した。既存の callable-only wrapper は non-callable
surface を受け取らないまま維持し、semantic table を渡す新しい materializer だけが nominal /
trait definition を扱う。

この checkpoint で追加した境界:

- `TypeCtx::checkpoint` / `rollback` を使い、途中 entry が reject した場合は named table、
  nominal identity、constructor binding、semantic table を汚さない。
- `Struct` / `Enum` は `PublicNominalTypeIdentity` を必須 authority とし、同名型が既にある場合は
  stable identity が一致するときだけ再利用する。
- artifact 内の identity はそのまま信頼しない。`Struct` / `Enum` は surface の type parameter、
  field / variant、type term から nominal definition hash を再計算し、`TypeCtx` へ登録する直前にも
  materialized `TypeKind` の hash と照合する。trait も capability / method surface から
  definition hash を再計算し、重複 method 名は fail-closed に拒否する。
- `PublicTypeTerm::Named { identity: Some(...) }` は、predeclare 済み nominal type と stable identity が
  一致する場合だけ `TypeId` へ戻す。
- `PublicTypeTerm::Apply` は base / args を materialize したうえで `TypeCtx::apply` へ戻す。
- `PublicTraitSurface` は `PublicTraitIdentity` を必須 authority とし、`TraitInfo` に
  `TraitStableIdentity` を保持して idempotent reuse と同名 trait conflict を区別する。
- trait method surface 内の `TraitSelf` は trait materializer 内だけで `self_ty` へ戻す。

この変更により、callable が後続の `Struct` entry を参照していても、predeclare pass により
definition order に依存せず named type を解決できる。enum variant constructor と struct constructor
binding も source typecheck と同じ `Env` 経路へ staging する。

### 2026-06-01 `.neplmeta` semantic impl materializer checkpoint

`.neplmeta` semantic table 付き materializer は、`PublicImplSurface` を validated `ImplInfo`
として staging し、全 entry が成功した後にだけ impl registry と capability target を更新するようにした。
artifact MVP gate の `UnsupportedImplLookup` は外し、impl surface を含む artifact も projection / preflight
を通過できる。

この checkpoint で追加した境界:

- `PublicImplKind::Trait` だけを受け入れる。inherent impl は source 側でも unsupported なので
  fail-closed に残す。
- trait application と generic bound は `PublicTraitRef.identity` を必須 authority とし、
  `TraitInfo.stable_identity` と source path / name / arity / definition hash が一致する場合だけ戻す。
- impl target と trait argument は既存の `TypeTermMaterializer` を使い、stable nominal identity 付き
  `Named` / `Apply` / binder-indexed generic parameter から復元する。
- duplicate impl は、trait application と target type pattern が重なる場合に拒否する。同一 impl の
  再投影は `AlreadyPresent` として扱う。
- `Clone` / `Copy` / `Drop` capability target は staging 中に直接更新せず、全体成功後にだけ
  `TypeCtx` へ登録する。`Copy` impl は対応する `Clone` impl がなければ拒否し、`Drop` impl は
  copyable target と重なる場合に拒否する。

まだ残る範囲:

- field accessor callable と function value identity は引き続き stable callable identity の別 issue に残す。
- re-export projection は target artifact chain と衝突判定が必要なので fail-closed のままにする。
- materialized dependency surface を実際の import / prelude typecheck body skip 経路へ接続し、base
  compile time の実測を更新する必要がある。

### 2026-06-01 `.neplmeta` typecheck materialized surface input checkpoint

typecheck driver に、artifact projection 済みの dependency `TypedPublicSurfaceTable` を source body
検査前に注入する入力境界を追加した。これにより、loader / web が `.neplmeta` store から得た surface を
通常の source declaration と同じ `Env` / nominal table / trait table / impl table へ戻せる。

この入力は `module_path` と `file_id` を必須にする。`SourceMap` 上の `file_id` が同じ path を指して
いない場合は fail-closed に拒否する。import visibility は binding span の file id に依存するため、
dependency body を省略しても target file slot は現在 compile の `SourceMap` に存在しなければならない。

materialized dependency は名前解決と semantic registry へ使うが、現在 module の public signature /
public surface には含めない。`StructInfo` / `EnumInfo` / `ImplInfo` に origin span を持たせ、typecheck
結果の public artifact 生成時に materialized file id を除外する。これにより、root module の
`.neplmeta` が import した dependency API を local export として再配布する事故を避ける。

この checkpoint でも body skip は完了していない。直接呼び出される dependency function body を
codegen 入力から落とすには、`.neplobj` / codegen fragment 相当の authority か、依存 callable を
使う case では source fallback する判定が必要である。次の実装では prelude capability のように
型検査だけで必要な surface から先に materialize し、実行 body が必要な dependency は安全側に戻す。

### 2026-06-01 `.neplmeta` compiler public-interface pipeline checkpoint

compile / prepare pipeline に `PublicInterfaceArtifactInputs` を追加し、dependency public surface hash、
root module surface、materialized dependency surface を同じ authority として渡せるようにした。
従来 wrapper は空の materialized surface を渡すため、通常 compile の挙動は変えない。

materialized surface がある場合、typecheck は `typecheck_with_materialized_public_surfaces` を通る。
ただし `.neplmeta` は関数本体、Resource IR proof、backend code fragment を持たないため、
`neplmeta$...` stable callable symbol が HIR の直接呼び出し、function value、`memo_call` value、
indirect call の codegen 入力へ到達した場合は `backend.codegen.materialized_function_body_missing`
で fail-closed に止める。loader / Web session はこの診断を source fallback の根拠にできる。

loader の edge probe には `target_file_id` を追加した。`MaterializedPublicSurfaceInput` は
`module_path` と `file_id` の一致を検査するため、後続の Web bridge は loader が現在 compile の
`SourceMap` へ登録した target file slot をそのまま使える。

この checkpoint で安全に扱える範囲:

- import / prelude の dependency surface が名前解決や capability 判定にだけ必要な場合。
- selected callable body が不要で、root HIR / Resource IR / codegen が root source 内で閉じる場合。
- selected callable body が必要になった場合に、`.neplobj` 実装まで source fallback へ戻す場合。

まだ残る範囲:

- Web / loader が artifact projection 成功時に dependency body merge を省く本線接続。
- selected materialized callable を source fallback ではなく `.neplobj` / `.nepllink` で解決する backend artifact。
- `memo_call` / function value identity へ `.neplmeta` callable を入れるための stable function identity と Resource proof。

### 2026-06-01 `.neplmeta` Web materialized body-skip checkpoint

Web `CompilerSession` の warm artifact store から、stdlib import / prelude edge の `.neplmeta`
projection 成功結果を loader の merge 境界へ渡すようにした。loader は対象 module を current
`SourceMap` に登録し、edge probe から得た `file_id` と `TypedPublicSurfaceTable` を
`MaterializedPublicSurfaceInput` として返す。typecheck はこの input を source declaration より前に
semantic registry へ materialize する。

この checkpoint の contract は次である。

- `#import` / prelude edge は `.neplmeta` projection が成功した場合だけ dependency root item merge を省く。
- `#include` は translation-unit merge として扱い、artifact boundary にしない。
- projection callback が `None` を返した edge は従来の source merge へ戻る。
- materialized compile attempt が失敗した場合は、Web session が full source load / compile を再実行する。
- metadata-only callable が direct call、function value、memoized function value、indirect call の codegen 入力へ到達した場合は、`.neplobj` が入るまで source fallback へ戻す。

この段階では dependency source の parse / module surface 計算を完全には省いていない。loader は
edge probe の `SourceMap` file id、source identity、dependency public surface hash を安定に作るため、
target source を読み込む。したがって base compile time を 0.5 秒未満へ近づける本命は、
bundled stdlib `.neplmeta` preseed、`.neplproof`、`.neplobj` / `.nepllink`、および loader が
source body parse を行わず interface artifact から file slot と surface を復元する次段である。

現時点で改善されるのは、warm store がある Web session で dependency body を root AST に混ぜず、
typecheck / Resource IR / codegen が root source と materialized semantic surface だけで進める case である。
projection や body 欠落が残る case は source fallback に戻るため、正確性は fail-closed に保たれる。

### 2026-06-01 `.neplmeta` materialized compile fallback stats checkpoint

Web `CompilerSession` に、`.neplmeta` projection 成功後の materialized compile attempt と
source fallback を分けて観測する counter を追加した。既存の artifact store stats は
missing / compatibility / payload / projection reject を扱う。今回の counter は、その後段で
`MaterializedPublicSurfaceInput` を実際に compile pipeline へ渡したか、渡した結果として source
fallback へ戻ったかを記録する。

追加した観測値:

- `nepl_meta_materialized_compile_attempts`
- `nepl_meta_materialized_compile_attempted_surfaces`
- `nepl_meta_materialized_compile_accepts`
- `nepl_meta_materialized_compile_source_fallbacks`
- `nepl_meta_materialized_compile_source_fallback_successes`
- `nepl_meta_materialized_compile_source_fallback_failures`
- `nepl_meta_materialized_compile_body_missing_fallbacks`
- `nepl_meta_materialized_compile_last_outcome_code`
- `nepl_meta_materialized_compile_last_fallback_reason_code`
- `nepl_meta_materialized_compile_last_attempted_surfaces`

fallback reason は enum code として公開する。`MaterializedFunctionBodyMissing` は、`.neplobj` /
`.nepllink` が未実装であるため source fallback へ戻る正常な安全側経路である。その他の core error も
現段階では correctness のため source fallback へ戻すが、performance 判断では別 reason として扱う。

compiled-output cache hit や stdlib overlay compile では materialized compile attempt を実行しないため、
last outcome は `NotAttempted` に戻す。これにより、前回 compile の fallback 状態を cache hit の最新状態と
誤読しない。

この checkpoint により、Node / Web の JSON から次を分けて計測できる。

- artifact が存在しないため projection へ進めない。
- projection は成功したが materialized compile が source fallback へ戻る。
- materialized compile が fallback なしで通る。
- source fallback 自体が失敗している。

base `compile_ms` と warm edit `compile_ms` は、既存の timing JSON とこの counter の delta を組み合わせて
読む。`loader_cache_stats_json` 側に elapsed time を重複保持せず、処理時間と compiler-internal reason を
別の authority として扱う。

### 2026-06-01 `.neplmeta` materialized compile performance report checkpoint

Node runner の `timing` に `compiler_session_stats` を追加し、`compiler_session_cache_before` /
`compiler_session_cache_after` の累積値から、この compile で増えた materialized compile counter だけを
delta として固定するようにした。これにより、長寿命 `CompilerSession` の累積 counter を performance
report 側で直接集計してしまう誤読を避ける。

`compiler_session_stats.materialized_compile` で扱う delta は次である。

- `attempts`
- `attempted_surfaces`
- `accepts`
- `source_fallbacks`
- `source_fallback_successes`
- `source_fallback_failures`
- `body_missing_fallbacks`

`compare_git_versions.js` は `compile_ms` / `run_ms` / `duration_ms` と同じ revision summary に
materialized compile delta summary を追加する。Markdown report では `Materialized Compile` table と
`Materialized Compile Delta from first ref` table を別に出し、base / warm edit compile time と
source fallback 率を同じ report で確認できる。

この report は `.neplobj` / `.nepllink` の実装ではない。目的は、`.neplmeta` projection は成功したが
body がないため source fallback した compile を、issue / benchmark / CI の JSON から継続的に見えるように
することである。特に `body_missing_fallbacks` が増える case は、`.neplobj` の最初の対象候補である。

### 2026-06-01 `.neplobj` candidate surface counter checkpoint

`CompilerSession.loader_cache_stats_json()` に `nepl_obj_candidate_body_missing_surfaces` と
`nepl_obj_candidate_last_body_missing_surfaces` を追加した。これは `MaterializedFunctionBodyMissing`
を理由に source fallback した compile で、compile pipeline に渡された materialized public surface 数を
`.neplobj` candidate として数える counter である。

この counter は function body や `TypeId`、`Span`、`SourceMap` を保存しない。`.neplmeta` projection
success 後に「いくつの materialized surface が executable body を要求したか」だけを観測する。symbol
単位・direct call / function value / memo_call / indirect call 単位の分類は、`.neplobj` stable link
symbol と backend fragment key を設計してから追加する。

`run_test.js` と `compare_git_versions.js` は、この counter も compile 単位の delta として扱い、
Markdown report では `body_missing_candidate_surfaces_delta_sum` として出す。これにより、compile
attempt 数ではなく、`.neplobj` が解決すべき surface 数を base / warm edit compile time と同じ report
で追える。

同じ checkpoint で、`nodesrc/bench_materialized_compile_fallbacks.js` を追加した。この script は単一
`CompilerSession` に対して cold base、warm store probe、body edit candidate、body edit repeat を順に
compile し、各 run の `compile_ms`、materialized compile delta、`resource_typecheck` /
`resource_static_check` / `wasm_codegen` の stage timing を JSON に出す。

通常の doctest suite は worker 分散と fixture 順序により session の温度が混ざる。したがって、
`.neplobj` candidate surface の実測は専用 benchmark script で行い、CI regression は helper と stats
shape を固定する。性能値そのものを brittle な閾値として test に埋め込まない。

### 2026-06-01 materialized fallback diagnostic detail checkpoint

`NeplMetaMaterializedCompileFallbackReason::OtherCoreError` は互換性のため残したまま、
`CompilerSession.loader_cache_stats_json()` に
`nepl_meta_materialized_compile_last_fallback_diagnostic_code` を追加した。これは materialized
compile attempt が source fallback へ戻る直前の primary `DiagnosticCode::as_str()` であり、
`OtherCoreError` の中身を message 文字列ではなく compiler-owned typed diagnostic registry で観測する。

`nodesrc/bench_materialized_compile_fallbacks.js` は per-run の
`materialized_compile.last_fallback_diagnostic_code` と summary の
`materialized_fallback_diagnostic_code_counts` を出す。これにより、`.neplobj` body missing と
typecheck materializer reject を同じ fallback 数に丸めず、次に直すべき materializer authority を
JSON だけから判断できる。

同じ checkpoint で、`PublicSurfaceMaterializeRejectReason` を coarse な
`type.public_surface.materializer_rejected` ではなく、`TypeDiagnosticCode` の stable code へ写す
helper を追加した。実測では `tmp/materialized-fallback-detail-20260601.json` の warm compile
3 回すべてが `type.public_surface.materializer.field_accessor_unsupported` であり、
`MaterializedFunctionBodyMissing` ではなかった。したがって次の root gap は `.neplobj` ではなく、
field accessor callable surface を current session の `Env` / HIR lowering boundary へ安全に
materialize することである。

ただし、`.neplmeta` 由来 field accessor callable は通常 callable と同一視しない。field accessor は
Resource / SourceCapability / HIR lowering と接続する compiler-owned callable なので、stable link
symbol や signature が一致しても、`get` / `get_ref` / `put` の accessor kind を保持したまま
`BindingKind::Func.field_accessor` として復元しなければならない。`@func`、`memo_call`、
indirect call への function value identity はこの checkpoint では広げない。

## 2026-06-01 field accessor materializer checkpoint

`PublicCallableSurface.field_accessor` を `.neplmeta` materializer で `FieldAccessorKind` へ戻し、
`BindingKind::Func.field_accessor` として current session の callable environment へ復元した。

materializer は field accessor の HIR や field offset を作らない。型 authority は structured public
surface の function type と signature hash に置き、direct call 時の aggregate layout / selector 解決は
既存の selected call path と `field_apply` に委ねる。

`def_id` は引き続き `None` である。これは `.neplmeta` が public interface artifact であり、source body
や Resource proof / codegen fragment ではないためである。したがって direct call は field accessor
metadata で `get_field` / `get_field_ref` / `set_field` intrinsic へ落とせるが、`@func`、indirect call、
`memo_call @func` のような function value identity 必須経路は fail-closed のままにする。

`tmp/materialized-field-accessor-20260601.json` の実測では、warm compile 3 回の
`type.public_surface.materializer.field_accessor_unsupported` は消えた。次の blocker は
`type.public_surface.materializer.callable_rejected` であり、`.neplobj` body missing にはまだ到達して
いない。したがって次 checkpoint は callable reject の詳細 code 化であり、missing link symbol、name
mismatch、callable type mismatch、arity mismatch、effect mismatch、signature hash mismatch を同じ
coarse diagnostic に畳み込まない必要がある。

## 2026-06-01 callable reject diagnostic detail checkpoint

`PublicSurfaceMaterializeRejectReason` の callable metadata reject を個別の stable
`TypeDiagnosticCode` に分けた。既存の `type.public_surface.materializer.callable_rejected` は互換性の
ため残すが、missing link symbol、link name mismatch、callable type expected、arity mismatch、
effect mismatch、signature hash mismatch はそれぞれ別 code として観測する。

`tmp/materialized-callable-reject-detail-20260601.json` の実測では、warm compile 3 回すべてが
`type.public_surface.materializer.callable.arity_mismatch` になった。これにより、次の root gap は
`.neplobj` body fragment ではなく、`PublicCallableSurface.arity` と
`PublicTypeTerm::Function.params` の生成 / materialize 境界であると分かった。

## 2026-06-01 field accessor wrapper arity checkpoint

`type.public_surface.materializer.callable.arity_mismatch` の根本原因は、通常 callable の arity
正規化ではなく、field accessor metadata の付与条件だった。`detect_field_accessor_fn` は
`#intrinsic "get_field_ref"` を使う関数を広く field accessor facade と見なしていたため、
`core/mem/types.region_token_size_ref` のような selector 固定 wrapper まで
`PublicCallableSurface.field_accessor=GetRef` として保存していた。

`core/field.get_ref` は `obj` と `idx` をそのまま intrinsic へ渡す 2 引数 facade である。一方、
`region_token_size_ref` は `token` だけを受け取り、selector `"size"` を内部で固定する 1 引数
wrapper である。この wrapper を `.neplmeta` materializer が `get_ref` ABI として復元すると、
`FieldAccessorKind::GetRef.argument_count() == 2` と `surface.arity == 1` が衝突する。

修正後の field accessor metadata は、intrinsic 名だけでなく、function parameter 数、intrinsic
argument 数、そして intrinsic argument が function parameter を同じ順序でそのまま渡していることを
確認した場合だけ付与する。selector literal を固定する wrapper は通常 callable として保存し、
selected body が必要になった場合は `.neplobj` が入るまで
`backend.codegen.materialized_function_body_missing` により source fallback へ戻す。

`tmp/materialized-field-wrapper-arity-20260601.json` の実測では、warm compile 3 回の
`type.public_surface.materializer.callable.arity_mismatch` は消えた。次の fallback diagnostic は
`backend.codegen.materialized_function_body_missing` であり、`materialized_body_missing_fallbacks_delta_sum=3`、
`neplobj_candidate_body_missing_surfaces_delta_sum=15` になった。したがって次の実装対象は
`.neplobj` / `.nepllink` の stable link symbol、selected callable body hash、generic instantiation
hash である。

## 2026-06-01 body-missing skip checkpoint

`.neplobj` がまだ存在しない状態で、同じ `CompilerSession` が同じ dependency edge を毎回
`.neplmeta` materialized compile へ投機投入すると、必ず `MaterializedFunctionBodyMissing` に戻る
edge でも compile pipeline と source fallback を二重に走らせてしまう。

そこで、`CompilerSession` に source-hash scoped な body-missing skip set を追加した。この set は
`target_module_path`、`target_source_key_hash`、`dependency_public_surface_hash` を authority にする。
materialized compile が `backend.codegen.materialized_function_body_missing` で source fallback した場合だけ、
その compile で投機投入された dependency edge を記録し、次回同じ source / dependency surface なら
`.neplmeta` projection を行わず通常 source merge へ戻す。

この checkpoint は `.neplobj` の代替ではない。むしろ `.neplobj` 未実装の間、確実に body が必要になる
edge を繰り返し失敗させないための negative cache である。`.neplobj` / `.nepllink` 実装後は、この同じ
key 境界で「skip」ではなく object fragment availability を見るように置き換える。

追加した観測値:

- `nepl_meta_body_missing_skip_entries`
- `nepl_meta_body_missing_skip_hits`
- `nepl_meta_body_missing_skip_stores`
- `nepl_meta_body_missing_skip_stale_entries`
- `nepl_meta_body_missing_skip_last_hits`
- `nepl_meta_body_missing_skip_last_stores`

`tmp/neplmeta-body-missing-skip-20260601.json` では、`materialized_body_missing_fallbacks_delta_sum` が
3 から 1 へ、`neplobj_candidate_body_missing_surfaces_delta_sum` が 15 から 5 へ下がった。後続の body edit
2 回では `body_missing_skip_hits_delta_sum=10` となり、既知の body-missing edge を再度 materialized
compile へ渡していない。

一方で `body_edit_candidate` / `body_edit_repeat` の `compile_ms` はまだ 1 秒前後であり、これは
`.neplobj` body fragment や stdlib preseed artifact がまだないためである。次の根本対応は、direct call に
限定した selected callable `.neplobj` key schema と availability resolver を入れ、function value /
`memo_call` / indirect call は stable codegen artifact と Resource proof が揃うまで fail-closed に残すこと。

## 2026-06-01 direct-call `.neplobj` key schema checkpoint

direct call に限定した `.neplobj` の最小 key schema を `NeplObjDirectCallKey` として
`nepl-core::artifact` に追加した。この checkpoint は object payload や linker ではなく、
後続の availability resolver / fragment store が使う invalidation boundary を先に固定する。

key に含める情報:

- compiler identity hash
- target / profile hash
- stdlib content hash
- backend feature set hash
- public callable stable link symbol
- materialized `neplmeta$...` symbol
- target source key hash
- selected callable body hash
- generic instantiation hash
- dependency public surface hash
- source capability policy hash
- private effect policy hash

`TypeId`、`Span`、`FileId`、`SourceMap`、typed HIR body は引き続き key に含めない。
`.neplmeta` の link symbol は公開 signature と source path を識別するが、body-only edit や
generic 具体化は識別しないため、`.neplobj` key では selected callable body hash と
generic instantiation hash を必須にする。

`materialized_callable_symbol_for_link_symbol` と `public_callable_link_symbol_stable_hash` を
`typecheck/public_surface.rs` の stable helper として公開し、materializer と `.neplobj` key が
同じ symbol 規則を使うようにした。これにより、`neplmeta$...` 文字列の生成規則が
materializer private helper に閉じず、object artifact 側でも同じ authority を参照できる。

この checkpoint でも function value、indirect call、`memo_call` は許可しない。`.neplobj`
availability resolver は次段階で direct call のみを対象にし、function value identity や
PrivateCache proof が必要な経路は stable backend representation と Resource proof が入るまで
fail-closed に維持する。

## 2026-06-01 materialized body-missing kind checkpoint

`.neplobj` availability resolver の直前準備として、materialized callable body-missing の
収集結果を codegen use kind 付きで保持するようにした。現在の診断 code は同じ
`backend.codegen.materialized_function_body_missing` のままだが、message では direct call、
function value、memoized function value を区別する。

この分類により、次段階の resolver は `HirExprKind::Call` の direct call だけを対象にできる。
`FnValue`、`MemoizedFunctionValue`、`CallIndirect` へ到達する materialized callable は、従来通り
source fallback / body-missing へ残す。これは `memo_call` の PrivateCache proof や function
value identity の stable backend representation を未実装のまま pure / object reuse 境界を
広げないための fail-closed 条件である。

## 2026-06-01 direct-call `.neplobj` availability input checkpoint

`PublicInterfaceArtifactInputs` に direct call 用 `.neplobj` availability 入力を追加し、
materialized callable body-missing dependency を diagnostic に変換する前に availability を見る
境界を作った。

現時点の実 compiler session はまだ key を渡さないため、通常の compile 挙動は fail-closed のままである。
これは意図的である。`.neplobj` key が存在するだけでは wasm / LLVM backend が実際に呼べる body
fragment を持つことを意味しない。caller は codegen fragment payload を同時に接続できる場合だけ
direct-call availability を渡す。

resolver は `HirExprKind::Call` の direct call のみを対象にする。同じ materialized symbol を持つ
`FnValue`、`MemoizedFunctionValue`、`CallIndirect` の callee は direct call key では解決しない。
この制約により、function value identity、indirect table lowering、`memo_call` の PrivateCache proof を
`.neplobj` direct-call MVP が暗黙に許可しない。

## 2026-06-02 direct-call `.neplobj` fragment payload checkpoint

2026-06-01 の key-only availability 入力は、backend payload を消費しないまま diagnostic だけを
消す危険があるため、backend 接続より先に使う設計としては破棄した。現 checkpoint では
`PublicInterfaceArtifactInputs` から `.neplobj` availability 入力を外し、body-missing diagnostic は
引き続き source fallback を要求する。

代わりに、次段階で backend/linker が実際に消費する payload 境界として
`NeplObjDirectCallFragmentArtifact` を追加した。key は
「どの direct-call body か」を判定する invalidation 境界であり、backend が実際に呼べる body を
提供しない。key だけで body-missing diagnostic を消すと、後続の wasm / LLVM backend は unknown
function symbol を受け取るため、`.neplobj` direct-call availability は key と backend payload を
不可分に扱う必要がある。

Wasm payload は `NeplObjWasmDirectCallFragment` として、params / results / function body bytes /
direct-call relocation を持つ。final module の function index は assembly 時点で決まるため、body
bytes だけに固定せず relocation を保持する。relocation は fragment 作成時に安定順へ正規化し、
同じ payload が入力順だけで別 cache key にならないようにした。重複 relocation offset と body 範囲外
offset は artifact 作成時点で拒否する。

次の実装では、source fallback / full compile で生成された `.neplobj` fragment payload を session
object store に保存したうえで、`PreparedProgram` / wasm codegen がその payload を function body set
と relocation map に登録する。diagnostic 抑制は、この backend 登録済み token が存在する場合にだけ
行う。

## 2026-06-02 wasm direct-call link-plan token checkpoint

`.neplobj` fragment payload を body-missing diagnostic 抑制へ直結させず、まず wasm backend の
function index 空間に登録できることを検査する link-plan token API を追加した。

`plan_neplobj_direct_call_fragments_for_wasm` は、現在の `HirModule` が持つ extern / user function と
同じ index 割当規則で `.neplobj` fragment を追加した場合に、次を確認する。

- fragment の materialized symbol が既存 function / import と衝突しない。
- fragment の backend feature set が現在 compiler の wasm backend feature set と一致する。
- direct-call relocation target が、既存 function または同じ plan 内の別 fragment として解決できる。

成功時は `NeplObjWasmDirectCallLinkPlanToken` を返す。この token は assigned function index、
direct-call key hash、fragment hash、resolved relocation を保持する。ただし、raw body bytes を
`CodeSection` へ投入し relocation を実際に patch する実装はまだないため、この token だけでは
body-missing diagnostic を消さない。次の段階では、token と function body insertion / relocation patch
を同じ backend 境界にまとめる。

## 2026-06-02 wasm direct-call fragment insertion checkpoint

`generate_wasm_with_neplobj_direct_call_fragments` は、通常の HIR function と `.neplobj`
direct-call fragment を同じ wasm module assembly に並べる。`plan_neplobj_direct_call_fragments_for_wasm`
で得た token を authority とし、fragment の signature は `TypeSection` / `FunctionSection` へ、
body bytes は `CodeSection::raw` へ投入する。

direct-call relocation は final module の function index を使って call immediate を patch する。
relocation offset は call opcode の直後であることを確認し、既存 immediate の LEB128 幅を読んでから
新しい function index の LEB128 encoding に差し替える。差し替えは後方から行うため、index が 127 を
超えて immediate が 1 byte から 2 byte 以上へ伸びる場合でも後続 relocation offset を壊さない。

`PublicInterfaceArtifactInputs` は `.neplobj` direct-call fragment payload を受け取れる。ただし、
body-missing diagnostic から外すのは `HirExprKind::Call` の direct call だけである。`FnValue`、
`CallIndirect`、`MemoizedFunctionValue` は function identity、table lowering、PrivateCache proof が
必要なので、direct-call fragment が同じ materialized symbol を持っていても引き続き fail-closed に
source fallback を要求する。

残る work は、source fallback / full compile で selected dependency body から fragment payload を
生成して same-session object store へ保存し、Web / loader の import edge が次回 compile でその store
を `PublicInterfaceArtifactInputs` へ渡すことである。

## 2026-06-01 selected body hash authority checkpoint

direct-call `.neplobj` key の `selected_callable_body_hash` は、Resource summary value cache が使う
Resource IR body hash と同じ authority から取得する方針にした。`resource_function_body_stable_hash`
を public Resource API として公開し、object key 側が `TypeId`、`Span`、一時値 ID、storage ID の
正規化規則を再実装しないようにする。

この hash は typed HIR body や raw source text ではなく、Resource IR の関数本文から作る。そのため
source body が `.neplmeta` で skip されている compile では自然には得られない。`.neplobj` store は
source fallback / full compile で selected dependency body を Resource IR まで下げた時点でこの hash を
作り、次回以降の direct-call availability key へ入れる。

raw wasm / LLVM body については、Resource IR body hash だけで本文文字列を識別しない。caller は
source capability policy hash、source key、backend feature set を合わせて `.neplobj` key を作り、
不確かな場合は fail-closed に通常 source fallback へ戻す。

## 2026-06-02 content-addressed stdlib dependency aggregate cache checkpoint

`LoaderSessionCache` は、通常の mutable provider session と、stdlib 全体の content hash を
namespace に使う bundled stdlib session を分けるようにした。Web `CompilerSession` は後者を使うため、
同一 session 内で同じ bundled stdlib source を読む場合、dependency aggregate public surface hash を
path/source hash から再利用できる。

通常の dependency aggregate key は module public surface hash と child aggregate hash を含むため、
hit 判定の前に依存先 source を parse して module public surface を再計算する必要がある。今回追加した
source-level key は source hash と child aggregate hash を含み、source token と依存閉包が同じ場合に
full parse / module clone を省く。さらに bundled stdlib 用の closed-source key は child aggregate hash を
含めないが、これは namespace が stdlib 全体の content hash であり、どれか 1 ファイルでも変われば
別 `LoaderSessionCache` へ分離される場合だけ有効にする。

この境界は `SourceMap`、typed HIR、Resource IR、codegen fragment を保存しない。保持するのは
path/hash keyed な public surface hash だけであり、stdlib overlay が指定された場合は従来どおり loader
cache と Resource summary artifact preseed を bypass する。mutable provider、test provider、
non-stdlib edge では child hash 付き key だけを使うため、依存先の公開面変更を closed-source key で
隠すことはない。

`tmp/artifact-closed-source-aggregate-cache-20260602-r3.json` では、materialized compile fallback bench が
次の結果になった。

- cold base `compile_ms=423`
- warm store probe `compile_ms=230`
- body edit candidate `compile_ms=206`
- body edit repeat `compile_ms=185`
- `materialized_body_missing_fallbacks_delta_sum=1`
- `neplobj_candidate_body_missing_surfaces_delta_sum=5`
- `body_missing_skip_hits_delta_sum=10`

同じ branch の direct probe では prewarm 側の dependency aggregate traversal が `dep_delta=838` まで
下がった。これは base compile の固定費を削る checkpoint であり、`.neplobj` object store、
bundled stdlib `.neplmeta` / `.neplproof` preseed、persistent `.nepl...` file format、
`memo_call` の PrivateCache proof を完了させるものではない。特に body edit の 0.1 秒目標にはまだ届かず、
次の root task は selected dependency body から `.neplobj` fragment を生成して same-session object
store へ保存する経路と、stdlib preseed artifact の初回 compile 投入である。

## 2026-06-02 same-session `.neplobj` direct-call fragment store checkpoint

`NeplObjDirectCallFragmentStore` を追加し、source fallback / full compile で得られる予定の
direct-call fragment payload を same-session で保持できる境界を作った。この store は永続
`.neplobj` codec ではなく、Web playground の `CompilerSession` が次回 compile で
`PublicInterfaceArtifactInputs` へ fragment slice を渡すための in-memory staging authority である。

lookup は materialized symbol だけでは hit しない。target/profile、stdlib content hash、target
source key、dependency public surface hash、source capability policy hash、backend feature set、
private effect policy hash、link symbol、空 generic instantiation hash をすべて確認する。source
capability policy がない edge では raw-memory / private-effect policy を比較できないため、通常
source fallback に残す。

Web `CompilerSession` は `.neplmeta` projection 後の `MaterializedPublicSurfaceInput` と loader の
edge probe を突き合わせ、store から一致する direct-call fragment だけを取り出して
`PublicInterfaceArtifactInputs::with_neplobj_direct_call_fragments` に渡す。body-missing negative skip は、
同じ edge context の object candidate が store に存在する場合には materialized probe を省略しない。
これにより、将来 fragment producer が入った後に古い body-missing skip entry が object hit を隠さない。

この checkpoint では fragment producer はまだ実装していない。`FnValue`、`CallIndirect`、
`MemoizedFunctionValue`、`memo_call` は引き続き direct-call fragment では解決しない。次の作業は、
source fallback / full compile 後に checked HIR / Resource IR body hash / wasm lowering から
`NeplObjDirectCallFragmentArtifact` を生成し、store へ保存することである。

## 2026-06-02 direct-call `.neplobj` leaf fragment producer checkpoint

full/source compile 成功時に direct-call `.neplobj` fragment を生成する最初の producer を追加した。
producer 入力は `NeplObjDirectCallFragmentExportRequest` であり、Web `CompilerSession` は
materialized public surface と loader edge probe から link symbol、target source key、
dependency public surface hash、source capability policy hash を渡す。

producer は request が空でない場合だけ動く。通常の base compile や `.neplmeta` materialized compile
attempt では追加の Resource IR lowering を行わない。request がある fallback/full compile では、final
codegen HIR を Resource IR へ lowering し、`resource_function_body_stable_hash` から
`selected_callable_body_hash` を作る。HIR そのもの、`TypeId`、`Span`、temporary id、`FileId` は
`.neplobj` key authority にしない。

現 checkpoint の export 対象は call relocation を持たない leaf function に限定する。direct call、
indirect call、function value、memoized function value、string literal、private-cache intrinsic、
generic function、raw wasm/LLVM body は fragment を返さず通常 source fallback と同じ扱いにする。
これは performance のために high-order function identity や PrivateCache proof を direct-call artifact で
代替しないための fail-closed 境界である。

Web `CompilerSession` は source fallback / full compile で生成された fragment を same-session
`NeplObjDirectCallFragmentStore` へ保存する。stdlib overlay compile では保存しない。次回 compile では
既存 store lookup が完全 key に一致した fragment だけを `PublicInterfaceArtifactInputs` へ渡すため、
body-missing skip cache と object availability が同じ edge context で衝突しない。

残件は、direct-call relocation を backend lowering plan から作る producer、generic instantiation hash、
string/data relocation、raw wasm/LLVM body の relocatable representation、persistent `.neplobj` codec、
そして function value / memoized value / `memo_call` PrivateCache proof の別 backend である。

## 2026-06-02 direct-call `.neplobj` relocation producer checkpoint

direct-call `.neplobj` producer を leaf function から public direct call を含む function へ広げた。
producer は request された public callable link symbol と、source fallback / full compile 済み HIR の
function name を対応付け、direct call の callee が同じ request set に含まれる場合だけ placeholder
function index を割り当てる。private helper、unrequested callable、builtin、trait dispatch は stable
link symbol を持つ `.neplobj` relocation target ではないため fail-closed にする。

relocation offset は wasm body bytes を後から scan して作らない。`lower_user` の HIR lowering を
`LoweredUserWasmInstructionStream` へ分け、producer は `wasm_encoder::Function` へ
`Instruction::Call(placeholder_index)` を emit する直前の `byte_len() + 1` を call immediate offset として
記録する。これにより offset は backend lowering が直接作った direct call と placeholder symbol の対応から
得られ、payload consumer の opcode check は壊れた cache payload を拒否する防衛境界に留まる。

`FnValue`、`MemoizedFunctionValue`、`CallIndirect`、string literal、private-cache intrinsic は引き続き
producer の対象外である。direct call の引数や block 内の nested expression にこれらが含まれる場合も
fragment は出さない。高階関数 identity、function table、memoized function の PrivateCache proof は
direct-call fragment では代替できないため、別 artifact / proof 境界として扱う。

追加 regression は次を固定する。

- `neplobj_wasm_export_produces_direct_call_relocation_fragment`: `dep_caller -> dep_target` の
  relocation を producer が作り、既存 `.neplobj` linker / patcher で実行結果が full body と一致する。
- `neplobj_wasm_export_rejects_direct_call_to_unrequested_target`: request set にない callee への
  direct call は fragment を生成しない。
- `neplobj_wasm_export_rejects_non_direct_call_leaf_boundaries`: direct call support 後も
  `FnValue` / `MemoizedFunctionValue` / `CallIndirect` / private-cache intrinsic / string literal /
  nested function value argument を拒否する。
- `neplobj_wasm_codegen_rejects_relocation_offset_without_call_opcode`: 永続 payload の offset が
  call immediate でない場合は consumer 側で `NeplObjDirectCallLinkInvalid` にする。

この checkpoint により direct-call dependency body は same-session `.neplobj` store から再利用できる
対象が leaf function から public direct-call graph へ広がった。ただし generic instantiation hash、
string/data relocation、raw wasm/LLVM body、function value / memoized function value backend、
`memo_call` PrivateCache mask proof、persistent `.neplobj` codec は未完了である。base compile_ms の
追加短縮には、これらと並行して bundled stdlib `.neplmeta` / `.neplproof` preseed を進める。

## 2026-06-02 Web `.neplobj` direct-call store regression checkpoint

Web `CompilerSession` の tree regression に、same-session `.neplobj` direct-call fragment store の
実運用経路を追加した。fixture は `core/char` の `char_utf8_cont_byte` を使い、同じ session で
小さな body edit を繰り返す。

1. 初回 compile は store が空なので `.neplobj` lookup を行わない。
2. 依存 artifact が projection された後、materialized compile が body-missing で source fallback へ戻る。
3. fallback full/source compile は selected dependency body から direct-call fragment を export し、store へ保存する。
4. 次の body edit では store lookup が hit し、fragment が `PublicInterfaceArtifactInputs` へ渡る。

regression は `nepl_obj_direct_call_fragment_store_lookup_hits` と
`nepl_obj_direct_call_fragment_store_lookup_fragments_returned` の増加を確認し、同時に
`nepl_meta_materialized_compile_body_missing_fallbacks` が増えないことを確認する。これにより、
body-missing negative skip cache が object hit を隠さず、direct-call `.neplobj` availability が
materialized compile を次の blocker まで進める境界を固定する。

この checkpoint は性能値そのものを更新するものではない。対象は regression coverage であり、
base compile_ms の追加短縮は bundled stdlib `.neplmeta` / `.neplproof` preseed と persistent artifact
codec の実装で継続する。

## 2026-06-02 `.neplmeta` / `.neplobj` fallback root-cause checkpoint

materialized dependency compile では、artifact が存在することだけで source body を置き換えてはいけない。
置き換える artifact set は、loader / typecheck / backend の各段階で同じ authority を満たす必要がある。
今回の checkpoint では、`.neplmeta` materializer と `.neplobj` direct-call store の実測 fallback から
次の不変条件を追加した。

1. import/prelude edge の materialization は edge-local に commit / rollback する。
   `load_file_with` の再帰中に見つかった child artifact は、parent edge の materialize 成否が決まるまで
   staging に置く。parent edge が materialize 成功した場合、child artifact は root input へ漏らさない。
   parent edge が source fallback した場合だけ、source body が必要とする child artifact を commit する。
   `include` は引き続き textual/source merge boundary であり、artifact edge にはしない。

2. source AST と materialized surface の二重登録は、`FileId` だけでなく source path boundary でも防ぐ。
   `.neplmeta` impl surface は `source_path` を持ち、同じ source path 由来の source impl が AST に残っても
   typecheck driver は二重登録しない。`FileId` は source map session-local であり、artifact replacement
   の永続 authority ではない。

3. nominal definition hash は stable nominal identity 付き placeholder を扱う。
   materializer は nominal type を先に placeholder として predeclare するため、field / variant type が
   `TypeKind::Named` placeholder を参照した状態で definition hash を検証する場合がある。このとき
   stable identity がある `Named` は backend scalar fallback ではなく stable key を hash する。

4. `.neplobj` direct-call lookup は relocation dependency closure を返す。
   caller fragment だけを返して callee fragment が欠ける状態は、link invalid ではなく object miss として
   source fallback に戻す。link validation は最後の fail-closed guard であり、store lookup の不足を
   diagnostic で丸めるための境界ではない。

5. raw wasm body は relocation-free leaf の場合だけ direct-call fragment にできる。
   call / call_indirect を含まない raw wasm body は、public direct-call relocation target として保存できる。
   raw wasm body が別 callable を呼ぶ場合は、raw wasm text から stable relocation を復元する設計が入るまで
   fail-closed に残す。

`tmp/materialized-raw-wasm-neplobj-20260602-rerun.json` では、`core/char` fixture の cold base
`compile_ms=387`、warm store probe `compile_ms=221`、body edit candidate `compile_ms=21`、
body edit repeat `compile_ms=21` だった。`materialized_non_body_missing_fallbacks_delta_sum=0` で、
残る fallback code は `.neplobj` artifact がまだない初回の `backend.codegen.materialized_function_body_missing`
だけである。

この結果は対象 fixture の warm edit を 0.1 秒未満に入れたことを示すが、永続 artifact codec や
bundled stdlib preseed の完成を意味しない。base compile_ms の一般化には、`.neplmeta` / `.neplproof`
preseed、persistent `.nepl...` codec、generic instantiation、string/data relocation、raw LLVM body、
function value / memoized function value backend、`memo_call` PrivateCache proof を別境界として進める。

## 2026-06-02 explicit `.neplmeta` preseed API checkpoint

Web `CompilerSession` に `preseed_nepl_meta_artifacts_for_source` と
`preseed_nepl_meta_artifacts_for_source_with_profile` を追加した。これは通常の
`prewarm_loader_cache_for_source` とは別の明示 API である。

`prewarm_loader_cache_for_source` は parser / loader query だけを warm し、typed HIR、
Resource IR proof、codegen fragment、dependency public surface artifact を作らない。新しい preseed
API は root source から import / prelude edge probe を収集し、対象 bundled stdlib module を
typecheck して `.neplmeta` public interface artifact を same-session `NeplMetaArtifactStore` へ保存する。
Resource IR static check や wasm codegen は実行しない。

この API を通常 compile path から暗黙に呼ばない理由は、preseed が typecheck を伴う重い処理だからである。
Web playground の表示上の compile 時間を下げるために前処理へ時間を移すだけでは性能改善ではない。JSON
bench では `preseed.elapsed_ms` と各 run の `compile_ms` を分け、永続 `.neplmeta` / IndexedDB / disk
cache が将来この preseed cost を compile 前に支払えるかを評価する。

`tmp/neplmeta-preseed-api-baseline-20260602.json` では preseed なしの `core/char` fixture が cold base
`compile_ms=438`、warm store probe `compile_ms=229`、body edit repeat `compile_ms=22` だった。
`tmp/neplmeta-preseed-api-enabled-20260602.json` では `preseed.artifact_count=40`、
`preseed.elapsed_ms=373`、preseed 後の cold base `compile_ms=261`、warm store probe
`compile_ms=23`、body edit repeat `compile_ms=23` だった。

これは、初回 compile 前に `.neplmeta` を用意できれば compile 本体は短くなることを示す。一方で、現状の
in-memory preseed は同じ処理を compile 直前に実行するため、総時間の目標達成ではない。次の根本対応は
persistent `.neplmeta` codec、bundled stdlib artifact embedding、`.neplproof` preseed、そして
`.neplobj` direct-call fragment の persistent store である。

## 2026-06-02 RPN cold base checkpoint

RPN は小さい user program だが、`std/stdio`、stack、integer parse、string builder、string trim、
byte buffer などを通じて現在の stdlib / Resource IR proof の固定費を代表する。したがって cold
base の基準 workload として `examples/rpn.nepl` を継続して使う。

native release の stage-only 測定では、`target\release\nepl-cli.exe --check -i examples\rpn.nepl
--target std --stdlib-root stdlib` が成功し、主な内訳は次だった。

| stage | elapsed |
|---|---:|
| `resource_typecheck` | 147ms |
| `resource_static_check` | 6172ms |
| `resource_initialized_moves` | 5320ms |
| `resource_initialized_i32_scalar_summaries` | 1330ms |
| `resource_initialized_raw_init_summaries` | 2315ms |
| `resource_initialized_function_checks` | 1609ms |
| `resource_owner_obligations` | 736ms |

release Web の cold base 測定では、`NEPL_RUN_TEST_SKIP_COMPILER_WARMUP=1` で
`nodesrc/run_test.js` を実行し、`compile_ms=9283`、`wasm_call_ms=8640`、`total_ms=9313`
だった。stage timing は `resource_typecheck=251.939ms`、
`resource_static_initialized_moves=6626.938ms`、`resource_static_owner_obligations=1455.416ms`、
`resource_static_check=8237.881ms` である。

per-function timing では、`str_trim` の final initialized function check が約 0.8-0.9 秒、
`sb_append_result` の i32 scalar summary が約 0.65-0.85 秒、`apply_op` / `dealloc_raw` /
`byte_builder_*` の raw-init summary が 0.1-0.5 秒台に分散している。`str_trim` の op timing では
loop / branch の clone / merge が支配的であり、path-sensitive alternatives の指数増殖ではなかった。
`NEPL_RESOURCE_PATH_REPLAY_DEBUG_FUNCTION=str_trim` では replay reason は出ていない。

この checkpoint で、enum variant projection の不可能 leaf を i32 scalar return fact 収集時に
削る試行を行ったが、RPN native release は `resource_static_check=7455ms` へ悪化した。したがって
この局所 pruning は採用しない。RPN cold base の支配コストは単一 leaf の過剰収集ではなく、
stdlib-heavy Resource proof を初回 compile で毎回構築する固定費である。

同じく、`str_trim` の branch / loop op timing から見えた `CellTable` / raw alias / slot table の
merge 前 clone を借用版 merge API へ置き換える試行も行った。focused tests は通ったが、
native RPN は `resource_static_check=7933ms`、`resource_initialized_moves=6949ms` へ悪化した。
clone の一部を消しても merge lattice そのもの、summary fixed-point、stable fact 再投影の固定費は残る。
この局所変更も採用せず、初回 proof 構築を preseed する方向を優先する。

2026-06-02 の追加 profiling では、`sb_append_result` の i32 scalar summary は op 伝播ではなく
return fact 収集の alias / offset / condition query が支配的だった。`I32ConditionQueryContext` の
memo は `Place` が既に安定順序を持つにもかかわらず `Vec` 線形探索だったため、同じ純粋 query の
探索空間を `BTreeMap` へ移した。この変更は fact の意味や検査境界を変えず、cache の検索構造だけを
置き換える。

Map 化後の native RPN では、`resource_initialized_i32_scalar_summaries` が `1421ms` / `1189ms`、
`resource_static_check` が `7389ms` / `7053ms` だった。filter timing では `sb_append_result` の
`collect_facts` が `684ms` / `159ms` から `374ms` / `109ms` へ下がった。ただし raw-init summary、
`str_trim` の initialized function check、owner summary の固定費は残っており、base compile 0.5 秒
未満には届かない。これは短期の探索構造改善として採用するが、根本対応は引き続き bundled /
persistent `.neplproof` preseed と stdlib prechecked artifact である。

2026-06-02 の追加 followup では、`str_trim` と `apply_op` の op timing をさらに分解した。`str_trim`
は final initialized check の branch / loop、`apply_op` raw-init summary は branch / match が支配的で、
path-sensitive replay の指数増殖ではなく、control-flow merge 内の `CellTable::availability_state_by`
固定費が見えていた。同関数は ancestor / descendant entry を一時 `Vec<CellStateEntry>` に clone し、
exact state と非 initialized flow を複数回走査していたため、exact non-initialized、ancestor、
descendant、raw-cell、exact initialized、initialized flow の優先順位を保ったまま allocation-free な
単一走査へ整理した。

同一 followup の clean baseline は `resource_static_check=7865ms`、`resource_initialized_moves=6756ms`、
`resource_initialized_raw_init_summaries=3113ms`、`resource_initialized_function_checks=2130ms` だった。
変更後の native release RPN stage-only 単独測定では `resource_static_check=6267ms` / `6381ms`、
`resource_initialized_moves=5208ms` / `5484ms` である。per-function timing では `str_trim` final check が
`895ms`、`apply_op` raw-init summary が `552ms` まで下がった。過去 checkpoint の最良値とは
測定揺れがあるため、この数値は同一 followup baseline からの改善として扱う。

remote/main の GUI 関連変更を取り込んだ main 上で release CLI を再ビルドした後の確認値は、
`resource_static_check=6937ms`、`resource_initialized_moves=5876ms`、
`resource_initialized_raw_init_summaries=2378ms`、`resource_initialized_function_checks=1905ms` だった。

次の根本対応は、`.neplproof` を persistent / bundled artifact として初回 compile 前に preseed
できるようにすることである。既存の `ResourceSummaryProofArtifact` は in-memory export/preseed と
fail-closed header 照合を持つが、`ResourceSummaryProofSnapshot` は serialization schema ではなく、
stable entry map も private である。したがって、disk / IndexedDB / build-time bundled artifact へ
進めるには、header first decode、stable entry codec、generic type-argument key、source capability
policy set hash、private effect policy hash を含む永続 codec を別 checkpoint として実装する必要がある。

## 2026-06-02 `str_trim` scan helper split checkpoint

remote/main の追加変更を取り込んだ後、RPN cold base を改めて同一 native release CLI で測定した。
stage-only run は `resource_static_check=7565ms` / `7455ms`、`resource_initialized_moves=6452ms` /
`6351ms`、`resource_initialized_raw_init_summaries=2980ms` / `2946ms`、
`resource_initialized_function_checks=2014ms` / `1974ms` だった。per-function timing では
`str_trim__str__str__pure` の final initialized function check が `1117ms` で最大の単一関数 cost
になっていた。

この checkpoint では、`str_trim` の意味を変えずに先頭側 scan と末尾側 scan を private helper へ
分けた。`str_trim_left_index` は先頭側の ASCII 空白が終わる byte index を返し、
`str_trim_right_index` は `start` より前へ戻らない末尾側の byte index を返す。public `str_trim`
は `len`、left scan、right scan、`str_slice` だけを接続する。これは Resource IR の検査規則や
stdlib の公開 API を変えず、1 つの関数に両方向 scan の loop / branch merge が集中する形を避ける
ための stdlib 構造改善である。

変更後の native release RPN stage-only 測定では、後続 run で `resource_static_check=5787ms` /
`5397ms`、`resource_initialized_moves=4742ms` / `4514ms`、
`resource_initialized_raw_init_summaries=2354ms` / `2394ms`、
`resource_initialized_function_checks=1029ms` / `921ms` だった。per-function timing では `str_trim`
が上位 50 件から外れ、残る上位は `apply_op` / `dealloc_raw` の raw-init summary と
`sb_append_result` の i32 scalar summary になった。

この変更は `str_trim` 由来の単一関数 cost を下げるが、RPN cold base 0.5 秒未満を達成するものでは
ない。subagent review でも、native `--check` はまだ `.neplproof` preseed を実際の stdlib proof
artifact から受け取れておらず、empty cache や局所 pruning に戻るべきではないと確認した。次の根本
対応は、final initialized pass、owner obligation、raw-init / i32 scalar summary を fail-closed な
`.neplproof` stable entry として bundled / persistent preseed へ接続することである。

## 2026-06-02 RPN operator / builder helper split checkpoint

`examples/rpn.nepl` を cold base の代表 workload として固定し、`str_trim` helper split 後の branch
で再度測定した。変更前の stage-only run は `resource_static_check=5870ms` / `5900ms`、
`resource_initialized_moves=4897ms` / `5011ms`、`resource_initialized_i32_scalar_summaries=1258ms` /
`1273ms`、`resource_initialized_raw_init_summaries=2560ms` / `2655ms`、
`resource_initialized_function_checks=1004ms` / `1015ms` だった。

per-function timing では、RPN 固有の `apply_op__Stack_T_i32_str...` raw-init summary が `611ms`、
`dealloc_raw` raw-init summary が `520ms`、`sb_append_result` i32 scalar summary が `441ms` だった。
`process_token` は `+` / `-` / `*` の分類後に `apply_op` へ渡すにもかかわらず、`apply_op` 内で
同じ token に対する `str_eq` chain を再実行していた。また、`sb_append_result` は空文字 fast path、
ByteBuilder append、error payload cleanup を 1 関数に集めており、return fact 収集の支配点になっていた。

この checkpoint では、RPN operator を `RpnOp` enum に分類する `operator_from_token` を追加し、
`apply_op` は分類済み `RpnOp` を受け取るようにした。演算値そのものは pure helper
`apply_op_values` に分け、Stack owner の pop / push と i32 演算選択を同一関数へ集中させない。
StringBuilder 側は `sb_append_non_empty_result` と `sb_byte_builder_error_text` を追加し、
public `sb_append_result` は空文字 fast path と非空 append path の接続に絞った。

変更後の native release RPN stage-only run は `resource_static_check=5372ms` / `4927ms`、
`resource_initialized_moves=4340ms` / `4013ms`、`resource_initialized_i32_scalar_summaries=1135ms` /
`1090ms`、`resource_initialized_raw_init_summaries=2309ms` / `2047ms`、
`resource_initialized_function_checks=824ms` / `801ms` だった。per-function timing run は計測出力の
overhead を含み `resource_static_check=5453ms` だったが、hot function の内訳は次の通りである。

```text
RPN cold base static check
  resource_static_check: 4927-5372ms in stage-only run
    resource_initialized_moves: 4013-4340ms
      resource_initialized_raw_init_summaries: 2047-2309ms
        dealloc_raw raw-init summary: 498ms
        apply_op raw-init summary: 426ms
        byte_builder_push_bytes_ref raw-init summary: 232ms
        byte_builder_push_u8 raw-init summary: 120ms
        byte_builder_reserve raw-init summary: 112ms
      resource_initialized_i32_scalar_summaries: 1090-1135ms
        sb_append_non_empty_result i32 scalar summary: 320ms
        byte_builder_reserve i32 scalar summary: 183ms
        byte_builder_push_bytes_ref i32 scalar summary: 118ms
        apply_op i32 scalar summary: 109ms
        byte_builder_push_u8 i32 scalar summary: 104ms
      resource_initialized_function_checks: 801-824ms
        dealloc_raw final initialized check: 166ms
        apply_op final initialized check: 99ms
        parse_u128_radix_digits_from final initialized check: 93ms
    resource_owner_summaries: 680-787ms
    resource_owner_obligations: 779-901ms
```

`apply_op` raw-init summary は、文字列 token を直接受けていた変更前の `611ms` から、分類済み
`RpnOp` を受ける形で `426ms` まで下がった。`sb_append_result` は public wrapper 自体が `3ms` へ
下がり、残る支配点は非空 append helper の `320ms` と ByteBuilder 系 helper に移った。

この改善は RPN source と StringBuilder の構造を Resource IR が扱いやすい粒度へ整理したものであり、
base compile 0.5 秒未満を達成するものではない。残る支配点は、`dealloc_raw` / ByteBuilder / Stack owner
flow の proof template と、stdlib-heavy Resource proof を cold start で毎回構築する固定費である。
次の根本対応は、actual `.neplproof` artifact を header-first / fail-closed に persistent / bundled
preseed へ接続し、final initialized pass、raw-init / i32 scalar summary、owner obligation を stdlib
prechecked proof として再利用できるようにすることである。

## 2026-06-02 RPN summary fixed-point index checkpoint

`examples/rpn.nepl` の cold base では、前 checkpoint 後も Resource summary 固定点の各反復で
summary slice から関数名索引を作り直していた。これは検査規則そのものではなく、同じ
`function -> summary position` 関係を何度も再構築する探索構造の固定費である。

この checkpoint では `SummaryNameIndex` を追加し、raw alias / i32 scalar / raw-init /
collection-slot / raw pointer / raw identity / owner summary の固定点中に、関数名から summary
位置を引く索引を保持するようにした。summary が追加・削除された場合だけ索引を更新し、各 summary
kind の `SummaryIndex` はこの索引を借用 view として使う。あわせて raw memory release requirement の
対象引数表を毎回 `Vec` にせず、静的 slice として返すようにした。

release CLI を再ビルドした後の native stage-only run は次の通りである。

```text
RPN cold base static check
  resource_static_check: 5899-6314ms
    resource_initialized_moves: 4776-5169ms
      resource_initialized_raw_init_summaries: 2572-2866ms
        recomputations: 156
        summaries: 80
      resource_initialized_i32_scalar_summaries: 1121-1227ms
        recomputations: 214
        summaries: 89
      resource_initialized_function_checks: 996-1028ms
      resource_initialized_raw_alias_summaries: 72-81ms
      resource_initialized_collection_slot_summaries: 2-3ms
    resource_owner_obligations: 903-986ms
      resource_owner_summaries: 794-859ms
      resource_owner_function_checks: 109-126ms
    resource_typecheck: 181-189ms
    resource_lowering + coverage: 61-65ms
    resource_effect_boundaries: 46-49ms
    resource_borrow_lifetimes: 37-42ms
```

同じ release rebuild 前の clean baseline は `resource_static_check=6622-6689ms`、
`resource_initialized_moves=5416-5455ms`、`resource_initialized_raw_init_summaries=2814ms`、
`resource_initialized_i32_scalar_summaries=1406-1428ms`、`resource_owner_obligations=1032-1061ms`
だった。したがって、今回の固定点索引化は proof の意味を変えずに、おおよそ 5-12% 程度の cold
static-check cost を削った。

関数別 timing run はログ出力 overhead を含むため総量比較には使わないが、残る支配点の順位は次の
通りだった。

```text
RPN per-function profiling hot spots
  raw_init_summary
    dealloc_raw: 1144ms in logging run
      internal raw-init stage: release_requirements 570ms in filtered sequential run
    apply_op: 515ms
      internal raw-init stage: release_requirements 380ms, variant_param_cells 117ms in filtered run
    byte_builder_push_bytes_ref: 489ms
    byte_builder_reserve: 472ms
    byte_builder_push_u8: 428ms
  i32_scalar_summary
    sb_append_non_empty_result: 377ms
    byte_builder_reserve: 216ms
    byte_builder_push_bytes_ref: 132ms
    apply_op: 117ms
    byte_builder_push_u8: 114ms
  resource_initialized_function_check
    dealloc_raw: 261ms
    apply_op: 224ms
    parse_u128_radix_digits_from: 152ms
```

この結果から、RPN cold base の次の根本対応は `dealloc_raw` / ByteBuilder / `apply_op` をさらに
関数分割することではなく、raw-init release requirement の control-flow replay と stdlib-heavy
Resource proof を初回 compile 前から利用できる `.neplproof` preseed に移すことである。空の
`ResourceSummaryValueCache` を CLI `--check` に接続する試行は miss/store overhead だけが増えて
悪化したため、persistent / bundled artifact なしの cache 接続は採用しない。

## RPN release requirement state-step checkpoint

2026-06-02 の follow-up では、`examples/rpn.nepl` の cold base を引き続き代表 workload とし、
raw-init summary 内の release requirement collector を詳しく測定した。

変更前の sequential baseline は `resource_static_check=5406ms / 5737ms / 5317ms`、
`resource_initialized_moves=4414ms / 4583ms / 4245ms`、
`resource_initialized_raw_init_summaries=2325ms / 2348ms / 2189ms` だった。
`dealloc_raw` の op timing では、release requirement collector が nested `Branch` body を
full `check_ops` で再実行しており、`branch count=15 total=1225ms max=319ms`、
`call count=144 total=647ms max=162ms` が支配的だった。

対応として、release requirement collector に control-flow state step を追加した。`Branch` と
`Match` は nested body の release obligation を再帰的に収集しつつ、後続 sibling op に必要な
`CellTable`、`CollectionSlotStateTable`、raw alias、function alias、pending realloc、variant
initialization だけを実際の Resource check と同じ規則で軽く進める。これにより、release obligation
収集と full state replay を重複させない。`CollectionSlotStateTable` は再帰呼び出しにも渡し、
control value transfer が collection slot lifecycle proof を失わないようにした。

あわせて、i32 scalar return fact の condition 収集に guard を入れた。raw alias 側に
i32 value condition を証明できる source がない場合、parameter / return condition の全候補探索を
行わない。直接定数と offset constant endpoint は保持するため、既存の condition proof 能力は
削っていない。

最終形の native release stage-only 3 run は次の通りである。

| stage | baseline median | final run1 | final run2 | final run3 | final median |
|---|---:|---:|---:|---:|---:|
| `resource_typecheck` | 160ms | 167ms | 148ms | 160ms | 160ms |
| `resource_initialized_i32_scalar_summaries` | 1104ms | 1116ms | 997ms | 1076ms | 1076ms |
| `resource_initialized_raw_init_summaries` | 2325ms | 918ms | 821ms | 935ms | 918ms |
| `resource_initialized_function_checks` | 922ms | 849ms | 765ms | 833ms | 833ms |
| `resource_initialized_moves` | 4414ms | 2958ms | 2648ms | 2917ms | 2917ms |
| `resource_owner_obligations` | 929ms | 847ms | 824ms | 831ms | 831ms |
| `resource_static_check` | 5406ms | 3941ms | 3595ms | 3878ms | 3878ms |

今回の範囲では、baseline 中央値 `resource_static_check=5406ms` から `3878ms` へ約 28% 下がった。
`resource_initialized_raw_init_summaries` は `2325ms` から `918ms` へ約 61% 下がり、今回の主効果は
raw-init release requirement の control-flow replay 削減である。

最終形の per-function timing は次の階層で残る支配点を示している。

```text
RPN final hot spots
  resource_initialized_i32_scalar_summaries=957ms
    sb_append_non_empty_result: 269ms
    byte_builder_reserve: 156ms
    byte_builder_push_bytes_ref: 96ms
    apply_op: 91ms
    byte_builder_push_u8: 82ms
  resource_initialized_raw_init_summaries=943ms
    apply_op: 189ms
    dealloc_raw: 164ms
    byte_builder_push_bytes_ref: 64ms
    byte_builder_reserve: 49ms
    vec_cleanup_copy_initialized_prefix: 29ms
  resource_initialized_function_checks=838ms
    dealloc_raw: 162ms
    parse_u128_radix_digits_from: 97ms
    apply_op: 93ms
    str_slice_result: 45ms
    eval_line: 41ms
```

`dealloc_raw` の最終 op timing では `branch count=6 total=288ms max=152ms`、
`call count=86 total=194ms max=75ms` になった。初期測定の `branch total=1225ms`、
`call total=647ms` からは大きく下がったが、Branch / call はまだ `dealloc_raw` raw-init summary の
主要部分である。

この checkpoint でも RPN cold base は 0.5 秒未満に届いていない。次の根本対応は、
`sb_append_non_empty_result` / ByteBuilder 系の i32 scalar condition proof の探索空間削減、
`dealloc_raw` / `apply_op` / Stack owner flow の Resource proof template 化、bundled / persistent
`.neplproof` preseed である。

## RPN i32 scalar variant projection filter checkpoint

2026-06-02 の follow-up では、RPN cold base の i32 scalar return fact 収集を再測定した。
baseline follow-up の native release stage-only 3 run は `resource_static_check=4056ms / 4247ms / 4008ms`、
`resource_initialized_i32_scalar_summaries=1127ms / 1129ms / 1027ms` だった。per-function では
`sb_append_non_empty_result=297ms`、`byte_builder_reserve=229ms`、`byte_builder_push_u8=131ms`、
`byte_builder_push_bytes_ref=125ms`、`apply_op=107ms` が i32 scalar summary の上位だった。

この支配点に対して、direct parameter condition の path 間 intersection を試したが採用しなかった。
1 path 目で候補を収集し、後続 path で候補だけを再検査する形にしたところ、
`resource_initialized_i32_scalar_summaries=1314ms / 1270ms / 1062ms`、
`resource_static_check=4411ms / 4318ms / 3725ms` へ悪化した。`sb_append_non_empty_result` の詳細では
1 本目の候補収集が `400ms` になっており、探索空間削減より別 context での再照会 cost が大きかった。

採用した変更は、return leaf 収集前に concrete variant から不可能な projection を除外する filter である。
`collect_i32_scalar_return_facts_for_value_suffix_cached_with_projection_filter` は、caller から受け取った
`projection_is_possible` を leaf enumeration に適用する。`function_i32_scalar_return_summary` では
後続の projection merge と同じ `state.concrete_variants.projection_is_possible` を渡すため、
`Result::Ok` が確定している path で `Err` payload の i32 fact 収集を始めない。不明な variant では
caller が true を返すため、従来通り fail-open の全探索になる。

変更後の native release stage-only 3 run は次の通りである。

| stage | previous checkpoint median | run1 | run2 | run3 | new median |
| --- | ---: | ---: | ---: | ---: | ---: |
| `resource_initialized_i32_scalar_summaries` | 1076ms | 1107ms | 1040ms | 993ms | 1040ms |
| `resource_initialized_raw_init_summaries` | 918ms | 894ms | 901ms | 893ms | 894ms |
| `resource_initialized_function_checks` | 833ms | 817ms | 839ms | 804ms | 817ms |
| `resource_initialized_moves` | 2917ms | 2897ms | 2850ms | 2763ms | 2850ms |
| `resource_owner_obligations` | 831ms | 804ms | 805ms | 792ms | 804ms |
| `resource_static_check` | 3878ms | 3834ms | 3793ms | 3684ms | 3793ms |

per-function timing はログ overhead と実行ぶれを含むため総量比較には使わず、残る支配点の順位として扱う。

```text
RPN variant-filter hot spots
  resource_initialized_i32_scalar_summaries=1133ms
    sb_append_non_empty_result: 317ms
    byte_builder_reserve: 189ms
    byte_builder_push_bytes_ref: 118ms
    apply_op: 112ms
    byte_builder_push_u8: 100ms
  resource_initialized_raw_init_summaries=1000ms
    apply_op: 195ms
    dealloc_raw: 164ms
  resource_initialized_function_checks=928ms
    dealloc_raw: 175ms
    parse_u128_radix_digits_from: 106ms
    apply_op: 100ms
```

この checkpoint は、静的検査規則を弱めずに不可能な sibling variant payload の探索開始を避ける小幅改善である。
RPN cold base median は `resource_static_check=3793ms` であり、0.5 秒未満にはまだ遠い。次の根本対応は、
native `--check` cold path に actual `.neplproof` preseed を接続し、stdlib-heavy proof を事前検査済み
artifact から読むことである。あわせて、`dealloc_raw` / `apply_op` / Stack owner flow の proof template と、
source 単位の i32 condition query memo を検討する。

## RPN source-specific proof gate checkpoint

2026-06-02 の follow-up では、native `--check` cold path と Web `CompilerSession` の `.neplproof`
利用境界を切り分けた。native `nepl-cli --check` は `check_module_with_source_map` を直接呼んでおり、
`ResourceSummaryValueCache` や in-memory `.neplproof` preseed を使っていない。一方、Web
`CompilerSession` は same-session の `ResourceSummaryProofArtifact` preseed / export slot を持つ。

空の `ResourceSummaryValueCache` を native CLI の check path に接続する試行は採用しない。RPN cold base
では preseed hit が無いまま stable key / replay probe の固定費だけが増え、`resource_static_check` は
`6761ms / 6849ms / 6816ms`、`resource_initialized_moves` は `4751ms / 4847ms / 4796ms`、
`resource_owner_obligations` は `1873ms / 1862ms / 1875ms` へ悪化した。native cold base で cache を
有効にするには、header-first / fail-closed な bundled または persistent `.neplproof` artifact が先に必要である。

採用した変更は、既存の proof を弱めない探索空間削減に限定した。

- raw-init release requirement では、summary application ごとに parameter alias list を一度だけ作る。
  `raw_address_suffix_after_address`、`ty: address_alias.ty`、`kind`、重複除去は従来通り維持する。
- i32 scalar return fact 収集では、対象 value の scalar alias に届く direct condition / value、relation、
  scale、offset が無い場合だけ condition 探索を省く。無関係な i32 fact が同じ graph にあるだけでは、
  cold leaf の全 condition を照会しない。
- relation / condition / parameter condition 収集で同じ `I32ConditionQueryContext` を共有し、同じ
  raw alias graph への alias / offset 到達性照会を繰り返さない。

formatter 後の native release RPN stage-only 3 run は次の通りである。

| stage | run1 | run2 | run3 | median |
| --- | ---: | ---: | ---: | ---: |
| `resource_typecheck` | 176ms | 146ms | 147ms | 147ms |
| `resource_initialized_i32_scalar_summaries` | 1035ms | 891ms | 860ms | 891ms |
| `resource_initialized_raw_init_summaries` | 976ms | 831ms | 896ms | 896ms |
| `resource_initialized_function_checks` | 883ms | 770ms | 757ms | 770ms |
| `resource_initialized_moves` | 2973ms | 2559ms | 2578ms | 2578ms |
| `resource_owner_obligations` | 865ms | 754ms | 839ms | 839ms |
| `resource_static_check` | 3979ms | 3433ms | 3538ms | 3538ms |

`origin/main` `9812d619` を取り込んで main へ merge した後の再測定では、
`resource_static_check=4083ms / 4177ms / 4082ms`、`resource_initialized_moves=3039ms / 3138ms / 3095ms`、
`resource_initialized_i32_scalar_summaries=1065ms / 1087ms / 1034ms`、
`resource_initialized_raw_init_summaries=998ms / 1030ms / 1037ms`、
`resource_initialized_function_checks=896ms / 934ms / 942ms`、
`resource_owner_obligations=894ms / 896ms / 848ms` だった。post-merge median は
`resource_static_check=4083ms` であり、RPN の基準値としてはこの値も併記する。

per-function timing は次の階層を示している。ログ overhead を含むため総量比較ではなく、残る支配点の順位として扱う。

```text
RPN cold base static check
  resource_static_check=3723ms
    resource_initialized_moves=2705ms
      resource_initialized_i32_scalar_summaries=929ms
        sb_append_non_empty_result: 268ms
        byte_builder_reserve: 173ms
        apply_op: 97ms
        byte_builder_push_bytes_ref: 72ms
        byte_builder_push_u8: 71ms
      resource_initialized_raw_init_summaries=890ms
        apply_op: 185ms
        dealloc_raw: 148ms
        byte_builder_push_bytes_ref: 59ms
        byte_builder_reserve: 46ms
        byte_builder_push_u8: 28ms
      resource_initialized_function_checks=811ms
        dealloc_raw: 151ms
        parse_u128_radix_digits_from: 91ms
        apply_op: 80ms
        str_slice_result: 53ms
        eval_line: 38ms
    resource_owner_obligations=881ms
```

2026-06-02 の follow-up remeasure では、HEAD 相当の native release `--check` でばらつきが大きい
`resource_static_check=6303ms / 4993ms / 4115ms` になった。run1 は
`resource_initialized_raw_init_summaries=2390ms` と `resource_owner_obligations=1400ms` が同時に
外れたため、RPN cold base は単発値ではなく 3 run 以上の stage hierarchy で見る必要がある。

同じ checkpoint の per-function timing 1 run は次の階層である。

```text
RPN cold base static check follow-up
  resource_static_check=4358ms
    resource_initialized_moves=3293ms
      resource_initialized_i32_scalar_summaries=1177ms
        sb_append_non_empty_result: 344ms
        byte_builder_reserve: 239ms
        apply_op: 103ms
        byte_builder_push_bytes_ref: 97ms
        byte_builder_push_u8: 90ms
      resource_initialized_raw_init_summaries=1053ms
        apply_op: 213ms
        dealloc_raw: 186ms
        byte_builder_push_bytes_ref: 75ms
        byte_builder_reserve: 49ms
        byte_builder_push_u8: 30ms
      resource_initialized_function_checks=959ms
        dealloc_raw: 168ms
        parse_u128_radix_digits_from: 111ms
        apply_op: 101ms
    resource_owner_obligations=923ms
      resource_owner_summaries=802ms
      resource_owner_function_checks=120ms
```

この follow-up で試した local cache / relevance filter は採用しない。

- `I32ConditionQueryContext` に condition candidate memo を追加する案は、`Place` key の `BTreeMap`
  固定費が hit 率を上回った。
- offset 由来 parameter condition の source 別収集を削る案は、focused test は通ったが RPN
  per-function で StringBuilder / ByteBuilder の i32 scalar cost が増えた。
- owner summary relevance filter は recomputation を `295 -> 267` へ減らしたが、filter 固定費が
  上回った。

したがって、RPN cold base を 0.5 秒未満へ進める主経路は、局所 memo ではなく bundled /
persistent `.neplproof` stable codec、native `--check` preseed、stdlib proof template、owner return
summary stable mirror である。

この checkpoint でも RPN cold base は 0.5 秒未満に届いていない。次の根本対応は、actual
`.neplproof` artifact を native `--check` cold path に接続し、stdlib-heavy proof を初回 compile 前から
preseed することである。owner return summary はまだ `.neplproof` payload に入っていないため、owner
return summary stable mirror も同じ優先度で進める。

2026-06-02 の proof-backed check gate checkpoint では、native cold `--check` に空の
`ResourceSummaryValueCache` を渡す経路を避けるため、cache 起動条件を Resource proof preseed の
有効性から分離した。追加した境界は `ResourceSummaryValueCacheActivation::OnlyAfterAcceptedPreseed`
である。same-session compile は既定の `Always` を使い、これまで通り空 cache から stable summary
entry を収集できる。一方、proof-backed check wrapper は `.neplproof` が missing / rejected / empty の
場合に Resource static check へ cache と context を渡さない。

この checkpoint の native release RPN stage-only 3 run は、変更前確認が
`resource_static_check=3785ms / 3359ms / 3563ms`、変更後が
`resource_static_check=3882ms / 3856ms / 3563ms` だった。通常 CLI path は baseline 範囲内だが、
RPN cold base はまだ秒単位であり、今回の変更は速度達成ではなく、persistent / bundled `.neplproof`
preseed を安全に接続するための root boundary である。

追加 regression は、既定 activation が従来の same-session cache instrumentation を維持すること、
`OnlyAfterAcceptedPreseed` では artifact が無い場合と matching header の empty artifact の場合に
cache instrumentation が起動しないことを固定する。次は disk / bundled artifact loader で
accepted usable entry を持つ `.neplproof` を渡し、RPN cold base の initialized moves と owner
obligations を初回から replay できるかを測る。

2026-06-02 の allocator / ByteBuilder reserve helper split checkpoint では、proof-backed check gate
後の RPN cold base をさらに関数構造から削った。開始時点の native release stage-only 3 run は
`resource_static_check=3888ms / 3600ms / 4158ms`、中央値 `3888ms` である。支配階層は次の通り。

```text
RPN cold base static check before allocator / ByteBuilder split
  resource_static_check: median 3888ms
    resource_initialized_moves: 2677-3107ms
      resource_initialized_i32_scalar_summaries: 903-1017ms
      resource_initialized_raw_init_summaries: 896-1020ms
      resource_initialized_function_checks: 801-993ms
    resource_owner_obligations: 788-895ms
```

per-function timing では `dealloc_raw` が raw-init summary と final initialized function check の両方に
残り、`byte_builder_reserve` が i32 scalar summary 上位に残っていた。対応として、allocator は
free list 挿入位置探索、link、next coalesce、prev coalesce を private helper へ分割し、public
`dealloc_raw` をこれらの接続へ縮めた。ByteBuilder は Empty storage grow と Owned storage grow を
private helper へ分け、public `byte_builder_reserve` を capacity 計算と storage branch 接続へ縮めた。

この変更は公開 API や検査契約を変えない。allocator は address-order free list、`ptr <= 0` no-op、
前後 coalesce、runtime ABI を維持する。ByteBuilder は失敗時の builder owner recovery、
capacity exceeded / out-of-memory / invalid operation の分類、旧 region token recovery を維持する。

変更後の native release stage-only 5 run は次の通り。

```text
RPN cold base static check after allocator / ByteBuilder split
  resource_static_check: 3134ms / 2819ms / 3054ms / 2801ms / 3018ms
    median: 3018ms
    resource_initialized_moves median: 2248ms
      resource_initialized_i32_scalar_summaries median: 783ms
      resource_initialized_raw_init_summaries median: 741ms
      resource_initialized_function_checks median: 651ms
    resource_owner_obligations median: 607ms
  native CLI elapsed by Measure-Command: about 3720ms
```

per-function timing では `byte_builder_reserve` i32 scalar summary が約 `166ms` から約 `36ms` へ下がった。
allocator 分割後、`dealloc_raw` は hot path 上位から大きく後退した。これは RPN cold base を
0.5 秒未満へ到達させるものではないが、stdlib の支配関数を小さい proof boundary へ分けると
Resource IR の固定点探索が実測で下がることを示す。

残る支配点は `sb_append_non_empty_result` / `byte_builder_push_bytes_ref` / `byte_builder_push_u8` の
i32 scalar と raw-init、`apply_op` / parse 系 function check、owner obligation である。次の主経路は
actual `.neplproof` preseed、stdlib proof template、owner return summary stable mirror である。

2026-06-02 の RPN Stack pop2 owner-flow checkpoint では、Stack owner flow の局所支配をさらに測った。
作業開始時点の native release stage-only 3 run は次の通りである。

```text
RPN cold base static check before Stack pop2
  resource_static_check: 2995ms / 2939ms / 2813ms
    median: 2939ms
    resource_initialized_moves: 2271ms / 2173ms / 2116ms
      resource_initialized_i32_scalar_summaries: 836ms / 744ms / 737ms
      resource_initialized_raw_init_summaries: 734ms / 700ms / 680ms
      resource_initialized_function_checks: 636ms / 666ms / 637ms
    resource_owner_obligations: 602ms / 640ms / 581ms
```

RPN の `apply_op` は従来、2 回の `pop_top` で Stack owner を移動し、途中の underflow path でも
owner recovery を個別に持っていた。`StackPop2` / `pop_top2` はこの 2 要素 pop を 1 つの owner
boundary にまとめる。公開 contract は、要素が 2 個以上なら lower / top と短くなった stack を返し、
2 個未満なら元の stack owner と `None` を返す、という形である。

変更後の native release stage-only 5 run は次の通りである。

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

ByteBuilder の reserved write helper 分割も同時に検討したが、helper 境界で owner recovery が
`resource.owner.use_after_move` になるため採用しなかった。これは静的検査を弱めて通す対象ではなく、
actual `.neplproof` と owner return summary stable mirror で関数境界を越えた証明再利用を入れるべき
箇所である。

この checkpoint は局所的な owner-flow API 改良であり、0.5 秒未満の cold base 目標を達成するものでは
ない。RPN の支配階層はまだ Resource IR initialized-state と owner obligation に残っているため、
persistent / bundled `.neplproof` preseed、stdlib proof template、owner return summary stable mirror を
主経路として継続する。

2026-06-02 の RPN `print_i32` allocation-free checkpoint では、RPN が error path と整数表示のために
`alloc/string/integer/format`、`StringBuilder`、`ByteBuilder` の proof graph を引き込んでいることを
再測定した。変更前の native release stage-only 5 run は次の通りである。

```text
RPN cold base static check before allocation-free print_i32
  resource_static_check: 3395ms / 2922ms / 2730ms / 2816ms / 2985ms
    median: 2922ms
    resource_initialized_moves median-near: 2098ms
      resource_initialized_i32_scalar_summaries median-near: 731ms
      resource_initialized_raw_init_summaries median-near: 640ms
      resource_initialized_function_checks median-near: 650ms
    resource_owner_obligations median-near: 703ms
  reachable functions: 307 kept=304
```

`stdlib/std/stdio/print.nepl` の `print_i32` は、`str` を確保せず digit byte を直接 stdout へ出す実装へ
変えた。`i32` 最小値の符号反転 overflow を避けるため、負数は負数のまま
`-1000000000` から `-1` までの divisor で上位桁から出力する。`examples/rpn.nepl` の stack count
error も、文字列連結ではなく固定文字列と `print_i32` の直接出力に変えた。

変更後の native release stage-only 5 run は次の通りである。

```text
RPN cold base static check after allocation-free print_i32
  resource_static_check: 1584ms / 1427ms / 1545ms / 1539ms / 1435ms
    median: 1539ms
    resource_initialized_moves median: 1041ms
      resource_initialized_i32_scalar_summaries median: 199ms
      resource_initialized_raw_init_summaries median: 365ms
      resource_initialized_function_checks median: 445ms
    resource_owner_obligations median: 394ms
  reachable functions: 271 kept=268
```

`origin/main` `c3066b8a` を merge した後の current tree では、
`resource_static_check=1433ms / 1535ms / 1504ms / 1460ms / 1500ms`、中央値 `1500ms` だった。
階層中央値は `resource_initialized_moves=1013ms`、`resource_initialized_i32_scalar_summaries=194ms`、
`resource_initialized_raw_init_summaries=355ms`、`resource_initialized_function_checks=435ms`、
`resource_owner_obligations=383ms` である。

これは compile check を skip する最適化ではなく、RPN が不要な stdlib formatting 実装を reachable
closure に含めないようにする構造改善である。0.5 秒未満目標にはまだ届いていないため、次の根本対応は
actual `.neplproof` preseed、stdlib proof template、owner return summary stable mirror である。

## safety contract

- call graph が静的に閉じない場合は、performance より正確性を優先して conservative-all にする。
- raw memory boundary / compiler memory type capability / owner token capability は source hash と path に結び、別 source へ再利用しない。
- generic substitution 後の Resource IR summary は、type argument hash を key に含める。
- cache hit しても、diagnostics は現在の source map へ再投影できる span だけを表示する。
- release artifact の stdlib hash が local stdlib より古い場合は、bundled stdlib を使わず FS stdlib override に戻す。
- session が保持する bundled stdlib hash と local stdlib content hash が一致しない場合は、mtime に関係なく FS stdlib override に戻す。

## 関連 issue

- [ISS-20260524T225852366Z-PER-PROGRAM-COMPILE-TIME-EXCEEDS-DEF-189918C5](../../issues/items/ISS-20260524T225852366Z-PER-PROGRAM-COMPILE-TIME-EXCEEDS-DEF-189918C5.md)
- [ISS-20260527T050120000Z-COMPILER-SESSION-STDLIB-PRECHECK-CACHE-A71E4C92](../../issues/items/ISS-20260527T050120000Z-COMPILER-SESSION-STDLIB-PRECHECK-CACHE-A71E4C92.md)
- [ISS-20260531T073211850Z-EXPRESSION-SUBTREE-INCREMENTAL-QUER-A91F3C2D](../../issues/items/ISS-20260531T073211850Z-EXPRESSION-SUBTREE-INCREMENTAL-QUER-A91F3C2D.md)
- [ISS-20260531T111205690Z-BINARY-INTERMEDIATE-ARTIFACTS-NEEDED-1C570649](../../issues/items/ISS-20260531T111205690Z-BINARY-INTERMEDIATE-ARTIFACTS-NEEDED-1C570649.md)
- [ISS-20260601T193116311Z-NEPLMETA-TRAIT-IMPL-MATERIALIZER-NEEDED-D3A0C2F1](../../issues/items/ISS-20260601T193116311Z-NEPLMETA-TRAIT-IMPL-MATERIALIZER-NEEDED-D3A0C2F1.md)
- [ISS-20260601T145100000Z-NEPLMETA-FIELD-ACCESSOR-MATERIALIZER-NEEDED-4F6A0C2B](../../issues/items/ISS-20260601T145100000Z-NEPLMETA-FIELD-ACCESSOR-MATERIALIZER-NEEDED-4F6A0C2B.md)
- [ISS-20260601T150700000Z-NEPLMETA-CALLABLE-REJECT-NEEDS-FINE-GRA-9D4F2A61](../../issues/items/ISS-20260601T150700000Z-NEPLMETA-CALLABLE-REJECT-NEEDS-FINE-GRA-9D4F2A61.md)
- [ISS-20260601T151600000Z-NEPLMETA-CALLABLE-ARITY-MISMATCH-BLOCK-5C8E2B91](../../issues/items/ISS-20260601T151600000Z-NEPLMETA-CALLABLE-ARITY-MISMATCH-BLOCK-5C8E2B91.md)
