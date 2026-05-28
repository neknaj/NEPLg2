---
id: ISS-20260527T050120000Z-COMPILER-SESSION-STDLIB-PRECHECK-CACHE-A71E4C92
title: "CompilerSession needs prechecked stdlib artifact and incremental query cache"
area: core
status: open
resolved: false
priority: P1
type: performance
created: 2026-05-27
updated: 2026-05-28
target: "nepl-core, nepl-web, nodesrc/run_test.js, stdlib"
---

# ISS-20260527T050120000Z-COMPILER-SESSION-STDLIB-PRECHECK-CACHE-A71E4C92: CompilerSession needs prechecked stdlib artifact and incremental query cache

## 概要

2026-05-27 の performance fix により、release WASM の最小 program は cold `compile_ms=231`、warm `compile_ms=5` まで下がった。一方で aggregate / generic / stdlib-heavy case を微小変更時に常に 10ms 未満へ収めるには、単発 compile API の warmup だけでは不足する。

## 対象

- `nepl-core`
- `nepl-web`
- `nodesrc/run_test.js`
- `stdlib`

## 根拠

- `target/debug/nepl-cli.exe --check -i tmp/minimal_perf.nepl --target wasm --stdlib-root stdlib`: elapsed 160ms、`resource_typecheck=5ms`、`resource_static_check=1ms`。
- `target/debug/nepl-cli.exe --check -i tmp/perf_alloc_probe.nepl --target wasm --stdlib-root stdlib`: elapsed 166ms、`resource_typecheck=6ms`、`resource_static_check=1ms`。
- `trunk build --release` 後の `nodesrc/run_test.js` minimal cold は `compile_ms=231`, `total_ms=257`, `stdlib_vfs_mode=bundled`。
- 同じ release artifact の minimal warm は `compile_ms=5`。
- aggregate warm は `compile_ms=22` で、10ms 未満にはまだ届いていない。
- direct WASM API の同一 source 連続 compile は初回 127ms、以後 3-5ms であり、release artifact と warm process の効果は確認できた。
- first checkpoint で `nepl-web` に `CompilerSession` を公開し、Node runner は session API を優先するようにした。release WASM session smoke では minimal warm `compile_ms=3`、aggregate warm `compile_ms=16`、cold minimal `compile_ms=160` だった。
- 同 checkpoint で bundled stdlib content hash を artifact に埋め込み、Node runner は hash が一致する場合にだけ bundled stdlib を使う。hash API のない旧 artifact では mtime fallback を維持する。
- Web playground worker と tutorial runtime も method 単位で `CompilerSession` を優先するようにし、full stdlib VFS object を通常の compile path から外した。
- Web playground の workspace compile request は read-only stdlib files と runtime data files を overlay VFS へ含めず、editable `.nepl` user source だけを送るようにした。WASI 実行用の full VFS snapshot は `runtimeVfsData` として compile overlay から分離した。
- Web terminal は compile 用 Worker を artifact URL 単位で保持するようにし、連続 build では同じ WASM instance / `CompilerSession` を再利用する。`neplg2 run` は compile だけ persistent Worker を使い、生成 wasm の実行は一回限りの runtime Worker に分離した。
- second checkpoint で `nepl-core::loader::LoaderSessionCache` を追加し、`CompilerSession` から bundled stdlib の raw parsed module cache を使うようにした。
- parsed stdlib cache は `cache version + stdlib namespace hash + canonical path + source hash + imported type arity hint hash` を key にし、cached AST は中立 `FileId` へ正規化してから保持する。cache hit 時は現在 compile の fresh `SourceMap` が割り当てた `FileId` へ再投影する。
- `SourceMap`、merged module 全体、`ImportResolution`、typed HIR、`TypeCtx` / `TypeId`、Resource IR summary、codegen fragment はまだ cache しない。これは stale span / stale source capability / stale `TypeId` を避けるためである。
- stdlib override または overlay が `/stdlib` 以下を差し替える場合は parsed module cache を bypass し、bundled stdlib artifact を local override へ混ぜない。
- `CompilerSession.loader_cache_stats_json()` と `nodesrc/run_test.js` の `timing.compiler_session_cache_before` / `timing.compiler_session_cache_after` により、warm compile が cache hit したかを JSON output から確認できるようにした。
- `trunk build --release` 後の release WASM で、minimal warm は `compile_ms=2`、aggregate first after warmup は `compile_ms=19`、同一 process / 同一 `CompilerSession` で aggregate を再compileした場合は `compile_ms=3` だった。aggregate 2 回目では cache hits が 4 から 9 へ増え、stdlib parsed module cache が実際に効いていることを確認した。
- subagent review 後、同一 path の source hash 変更、imported type arity hint 変更、forced stdlib VFS / stdlib overlay bypass、空 namespace cache の禁止を regression と API 境界で補強した。
- third checkpoint で `LoaderSessionCache` に source arity surface cache を追加した。これは local type arity hints、prelude/import/include/public re-export path、root-only default prelude 判定だけを持つ未型付け artifact であり、`FileId` / `Span` / `ImportResolution` / typed HIR / `TypeId` は保存しない。
- source arity surface cache は `cache version + stdlib namespace hash + stdlib root + canonical path + source hash` を key にし、public re-export 先の arity result は親 surface へ畳み込まない。依存先 source hash が変わった場合は、facade source が cache hit しても依存先 surface を再評価する。
- `CompilerSession.loader_cache_stats_json()` は `arity_surface_hits` / `arity_surface_misses` / `arity_surface_stores` も返すようになった。
- `trunk build --release` 後の same preloaded `CompilerSession` 実測では、minimal は `compile_ms=2`、aggregate first は `compile_ms=15`、aggregate second は `compile_ms=4` だった。aggregate second では parsed module hits が 8 から 13、arity surface hits が 8 から 14 に増えた。
- fourth checkpoint で source-directed loader prewarm を追加した。全 bundled stdlib file list の総なめは subagent review で却下し、root source の default prelude / prelude / import / include から到達する configured stdlib closure だけを prewarm する。
- 同 checkpoint で arity surface cache を configured stdlib path に限定した。user entry source や user VFS module は `arity_surface_bypasses` として観測し、長寿命 `CompilerSession` の cache value には残さない。
- Node runner は `selectStdlibVfsMode(meta) == "bundled"` の場合だけ `CompilerSession.prewarm_loader_cache_for_source()` を呼ぶ。forced stdlib VFS / fs override では skip reason を timing に残し、bundled stdlib prewarm は実行しない。
- prewarm 中の loader error は `compiler_session_prewarm_error` として観測し、通常 compile path は続行する。これにより、prewarm 専用の失敗が本来の compile diagnostic を隠さない。
- `trunk build --release` 後の source-directed prewarm 実測では、minimal は `compile_ms=3` / `prewarm_ms=1` / `wasm_call_ms=2`、aggregate first は `compile_ms=15` / `prewarm_ms=3` / `wasm_call_ms=11-12`、aggregate second は `compile_ms=4-5` / `prewarm_ms=1` / `wasm_call_ms=3-4` だった。aggregate first の total `compile_ms` はまだ 10ms 未満に固定できていないため、logical import graph / typed public surface / Resource IR summary cache が次の根本対応である。
- fifth checkpoint では、logical import graph の前段として loader の source arity surface を source import edge 表現へ広げた。edge は kind、resolved target path、visibility、import clause、source order を持つが、`FileId` / `Span` / `ImportResolution` / typed HIR / `TypeId` は持たない。
- parser に `parse_import_directive_parts` を追加し、loader の raw `#import` text parsing と parser の import clause parsing が分岐しないようにした。これにより、今後の graph cache で visibility / alias / selective import / merge clause を path-only edge に潰さない足場ができた。
- sixth checkpoint では、同じ root import surface に対する source-directed prewarm を `CompilerSession` 内で no-op にする guard を追加した。これは semantic cache ではなく、成功済み prewarm surface hash と warmed root count だけを保持する軽量な差分 compile guard である。
- root prewarm surface hash は loader cache version、canonical stdlib root、root default prelude state、`#no_prelude`、lexer error outcome、prelude/import/include edge の kind / resolved target path / visibility / import clause / source order を含む。root source body、local type arity hints、`FileId` / `Span` / `ImportResolution` / typed HIR / `TypeId` / Resource IR / codegen fragment は含めない。
- forced stdlib VFS、FS stdlib override、compile VFS 内の `/stdlib` overlay では bundled stdlib prewarm を呼ばない。prewarm failure では surface hash を記録せず、同じ source を次回再試行できる。
- `CompilerSession.loader_cache_stats_json()` は `prewarm_surface_hits` / `prewarm_surface_stores` を返す。body-only edit の同一 session 実測では aggregate case が `compile_ms=3` / `prewarm_ms=0` / `prewarm_surface_hits=1` / `wasm_call_ms=3` になった。
- seventh checkpoint では、typed public surface cache へ進む前の未型付け artifact として `module_public_surface_hash` を追加した。value は stdlib parsed module cache に付随し、`LoaderSessionCache` の hit/store/bypass 統計として観測する。
- `module_public_surface_hash` は public declaration header、logical import/prelude/include edge、public re-export、public extern、public alias target の local callable signature、public `noshadow`、trait capability、impl header / method signature を含む。docs、comments、function body、`FileId` / `Span` / `SourceMap` / `ImportResolution` / typed HIR / `TypeId` / Resource IR / codegen fragment は含めない。
- subagent review により、private import / prelude / include が public signature の名前解決に影響し得ること、public alias が private helper signature を公開し得ること、`noshadow` が cross-file binding behavior の一部であることを確認した。この checkpoint ではこの missing context を hash に反映し、当時未実装だった dependency aggregate hash と typed public signature table は後続 checkpoint へ分けた。
- `trunk build --release` 後の public surface hash checkpoint 実測では、aggregate first が `compile_ms=17` / `prewarm_ms=3` / `wasm_call_ms=14` / `public_surface_hash_hits=13`、aggregate second が `compile_ms=3` / `prewarm_ms=0` / `wasm_call_ms=3` / `public_surface_hash_hits=18`、body-only edit が `compile_ms=3` / `prewarm_ms=0` / `wasm_call_ms=3` / `public_surface_hash_hits=23` だった。
- eighth checkpoint では、`SourceImportEdge` と module `public_surface_hash` を畳み込む `root_dependency_aggregate_public_surface_hash_for_source_with_cache` を追加した。root source の import surface、canonical stdlib path、module public surface hash、child dependency aggregate hash を使う loader-level query であり、typed HIR / `ImportResolution` / `TypeId` / Resource IR / codegen fragment はまだ保持しない。
- dependency aggregate cache key は source body hash ではなく module public surface hash と child aggregate hash を使う。これにより dependency body-only edit では aggregate hit になり、public signature edit では invalidation される。non-stdlib dependency edge は provider で読まず conservative external hash として bypass する。
- `CompilerSession.loader_cache_stats_json()` は `dependency_aggregate_public_surface_hash_hits` / `misses` / `stores` / `bypasses` を返す。ただし 2026-05-28 の修正で、`CompilerSession.prewarm_loader_cache_for_source` は dependency aggregate hash を同期計算しない形へ戻した。これは当時 typed public surface / Resource IR summary cache がまだこの hash を消費しておらず、RPN のような stdlib-heavy source では compile 前 prewarm が private implementation graph を広く歩いて 120 秒 timeout するためである。現在も prewarm では計算せず、通常 compile path だけが Resource summary namespace key の入力として消費する。
- `trunk build --release` 後の dependency aggregate checkpoint 実測では、aggregate first が `compile_ms=17` / `prewarm_ms=3` / `wasm_call_ms=14` / `dependency_aggregate_public_surface_hash_hits=4` / `misses=5` / `stores=5`、aggregate second が `compile_ms=4` / `prewarm_ms=0` / `wasm_call_ms=4` / `prewarm_surface_hits=1`、body-only edit が `compile_ms=4` / `prewarm_ms=0` / `wasm_call_ms=4` / `prewarm_surface_hits=2` だった。
- 2026-05-28 checkpoint では、Web playground の通常 `trunk build` が Rust/WASM release artifact を作るように `Trunk.toml` の `[build].release = true` と HTML Rust asset の `data-cargo-profile="release"` を固定した。debug WASM を配布して compile が終了しないように見える経路を避ける。
- 同 checkpoint で、NEPL source profile の既定値を `debug` に固定した。compiler artifact 自身が release build でも、明示 profile がなければ `#if[profile=release]` は有効化しない。
- 同 checkpoint で `CompilerSession` に compiled-output cache を追加した。key は entry path、source、compile VFS、NEPL source profile、WAT comment mode であり、value は `CompiledWasm` だけである。`SourceMap` / typed HIR / `TypeId` / Resource IR summary / diagnostic span は保持しない。
- RPN の release WASM doctest 実測では、dependency aggregate を prewarm hot path から外した後、2 件とも完走した。同一 session の初回 compile は `compile_ms=8976` / `prewarm_ms=193` / `wasm_call_ms=8783`、同一 source 2 回目は `compile_ms=1` / `wasm_call_ms=0` / `compiled_output_cache_hits=1` だった。これは同一入力の再compile停止を避ける応急境界であり、初回 0.5 秒未満の目標は未達である。
- native release RPN static check は correctness review 修正後に `resource_initialized_i32_scalar_summaries=2012ms`、`resource_initialized_raw_init_summaries=2520ms`、`resource_initialized_function_checks=3730ms`、`resource_static_check=9202ms` だった。次は Resource summary namespace の下で未変更 dependency の Resource IR summary value を再利用する。
- `PreparedProgram` に `ResourceSummaryCacheNamespaceKey` を追加した。key は target / profile / typed public signature hash / dependency public surface hash option から決定的に作るが、現時点では Resource IR summary value の hit / store は行わない。`TypeId` / `Span` / `SourceMap` / typed HIR / Resource IR body / diagnostic span / codegen fragment は key に保存しない。
- namespace key checkpoint 後の native release RPN stage-only 測定は `resource_typecheck=121ms`、`resource_initialized_i32_scalar_summaries=1270ms`、`resource_initialized_raw_init_summaries=2187ms`、`resource_initialized_function_checks=3063ms`、`resource_static_check=7353ms`。key staging 自体は summary value reuse ではないため、初回 compile 0.5 秒未満はまだ未達である。
- session-backed bundled stdlib compile path では、loader の dependency aggregate public surface hash を `ResourceSummaryCacheNamespaceKey` へ渡すようにした。汎用 `CompileOptions` は増やさず、Web session path だけが明示 helper 経由で渡す。Web / Node prewarm hot path では引き続き dependency aggregate を同期計算しない。
- RPN の release WASM doctest 実測では、初回 `compile_ms=9095` / `prewarm_ms=210` / `wasm_call_ms=8884`、同一 source 2 回目は `compile_ms=1` / `wasm_call_ms=0` / `compiled_output_cache_hits=1` だった。prewarm 前後では dependency aggregate counter は増えず、compile 本体でだけ dependency aggregate counter が増えた。
- duplicate path dedup / string byte predicate checkpoint では、Resource IR summary value cache の前に安全な局所探索削減を追加した。これは cache value reuse ではなく、budget 超過時の完全重複 path-state replay と `str_trim` の Option branch 展開を減らすものである。
- subagent review では、既存 Resource summary struct が `TypeId` / `Span` を含むため、そのまま `CompilerSession` の長寿命 value に保存しないことを確認した。次 checkpoint の Resource summary value cache は namespace key に function body hash、generic type-argument hash、source capability policy hash、summary kind/version を足し、arena 非依存の stable mirror value だけを保存する。
- 同 checkpoint の native release RPN stage-only 測定は `resource_initialized_i32_scalar_summaries=1256ms`、`resource_initialized_raw_init_summaries=2549ms`、`resource_initialized_function_checks=3139ms`、`resource_static_check=7870ms`。初回 0.5 秒未満にはまだ届かない。
- Resource summary value cache の設計レビューを追加で行い、cache owner は `LoaderSessionCache` ではなく `CompilerSession` の別 field とする方針を確認した。初期 stable mirror は `RawCellInitializationFunctionSummary` 全体ではなく、`CollectionSlotLifecycleSummaryOp::DropTraversal` のような小さい summary kind から始める。
- RPN signed integer parse checkpoint では、Resource summary value cache の前に、`to_i128_radix` の signed-body `str_slice` と nested `Result` match を削除した。native release RPN stage-only 測定は best run で `resource_initialized_i32_scalar_summaries=1172ms`、`resource_initialized_raw_init_summaries=2239ms`、`resource_initialized_function_checks=1767ms`、`resource_static_check=6104ms`、`trunk build --release` 後の再確認で `resource_initialized_i32_scalar_summaries=1450ms`、`resource_initialized_raw_init_summaries=2647ms`、`resource_initialized_function_checks=1965ms`、`resource_static_check=7086ms`。これは局所探索削減であり、次 checkpoint の Resource summary value cache 方針は維持する。
- 追加 review では、RPN call graph pruning が conservative-all へ倒れているのではなく、monomorphized function 290 件のうち 287 件が実際に到達していることを確認した。次の大きな削減余地は import pruning ではなく、到達済み関数の Resource summary value reuse である。
- Resource summary value cache の初期 implementation boundary は `CompilerSession` の別 field とし、`LoaderSessionCache` には入れない。`LoaderSessionCache` は未型付け source / AST / loader surface の cache であり、Resource IR proof artifact と invalidation 境界を混ぜないためである。
- 初期 stable mirror は `CollectionSlotLifecycleSummaryOp::DropTraversal` と `ForallInitializedRange` に限定する。key は namespace key、function body hash、generic type-argument hash、source capability policy hash、summary kind/version を含める。value は stable summary place / projection / type key、known i32、expected type、element stride、`StateOnly` / `LoadedValueDrop` proof のような arena 非依存データだけにする。
- 初期実装では `CertifiedSlots`、`TransformRange`、`Event`、`Relocate`、return path facts、Merge / Loop にまたがる proof、`RawCellInitializationFunctionSummary` 全体、raw alias graph、`TypeId`、`Span`、`SourceMap`、typed HIR、diagnostic span を store しない。`expected_ty` や `LoadedValueDrop` proof 内の型も stable type key へ落とし、現在 compile の `TypeCtx` へ再投影できる場合だけ store する。
- metrics は compiled-output cache と分け、`resource_summary_value_hits` / `misses` / `stores` / `bypasses` と summary kind 別 counter を JSON timing から観測できるようにする。`resource_summary_value_hits` は逆投影可能な stable value の candidate hit であり、compile work skip の成果とは分ける。実 replay は `resource_summary_value_replay_*` counter で別に測る。
- implementation staging として `nepl-core::resource::ResourceSummaryValueCache` / `ResourceSummaryValueCacheStats` を追加し、`CompilerSession` が `LoaderSessionCache` とは別 field で所有するようにした。`loader_cache_stats_json()` は `resource_summary_value_*`、`resource_summary_value_replay_*`、`resource_summary_value_drop_traversal_forall_*` を返す。stable mirror value の store/hit は map MVP まで実装済みだが、現時点ではまだ fixed-point worklist の skip は行わない。
- bypass instrumentation checkpoint として、session-backed bundled stdlib compile path の compiled-output cache miss だけが `ResourceSummaryValueCache` を `nepl-core` の Resource initialized check へ渡すようにした。既存の `check_resource_initialized_moves` は stateless / CLI 用に残し、cache 付き経路は `check_resource_initialized_moves_with_summary_cache` へ分ける。
- 現時点では worklist 固定点が収束した後の最終 `CollectionSlotLifecycleFunctionSummary` に top-level op として残る `CollectionSlotLifecycleSummaryOp::DropTraversal` かつ `ForallInitializedRange` の候補を hit/store せず、`resource_summary_value_drop_traversal_forall_bypasses` として数える。return path facts や `Merge` / `Loop` 内の leaf は初期 MVP の store 対象外なので、この counter には含めない。これにより、stable mirror key/value を実装する前に、対象候補が実 workload の compile path でどれだけ出るかを観測できる。
- stable mirror conversion checkpoint として、`DropTraversal + ForallInitializedRange` を `ResourceSummaryStableDropTraversalForallValue` へ変換する型を追加した。型は `TypeId` ではなく `ResourceSummaryStableTypeKey` として保持し、無名 type variable や cycle のように arena slot へ依存する型は保存候補から外す。現時点では map への store/hit はまだ行わず、bypass counter も stable mirror へ変換できた top-level 候補だけを数える。
- stable mirror split checkpoint として、stable mirror 変換を `resource_summary_value_cache::stable_mirror` private submodule へ分け、cache owner の可視性を広げない形にした。store/hit 前の追加 review により、per-summary-value key は namespace key だけではなく canonical function identity、function body hash、function-local type parameter boundary、generic type-argument hash、source capability policy hash、summary kind/version を structured key として持つ必要があると明記した。`SummaryOffset::Unknown` は exact offset を再投影できないため stable mirror 変換で拒否し、nominal type は qualified module/path/definition identity が得られるまで store 対象にしない。
- structured key staging checkpoint として、`resource_summary_value_cache::key` private module に per-summary-value key 型を追加した。これは map store/hit ではなく、namespace hash、function identity、function body hash、function-local type parameter boundary hash、generic type-argument hash、source capability policy hash、summary kind/version を field として分けて保持するための足場である。追加 review により、function body hash と source capability policy hash が pipeline から渡るまで store/hit API は公開しない。
- source capability policy hash checkpoint として、`SourceMap::source_capability_policy_hash_for_file(file_id)` を追加した。これは Resource summary value key に入れる deterministic fingerprint であり、source capability を広く許可する query ではない。canonical path と source content hash を含め、同じ byte range の proof が別 source へ stale hit しないことを regression で固定した。source hash は caller に渡させず `SourceMap` 内の source text から計算する。
- function body hash staging checkpoint として、`ResourceSummaryStableTypeKey` を shared private module へ分け、`resource_summary_value_cache::body_hash` で `ResourceFunction` の stable body hash を作る足場を追加した。`Span` は hash せず、`TypeId` は stable type key へ変換し、temporary / block id は body 内 ordinal に正規化する。subagent review により `StorageId` は body だけでは安定 origin へ対応付けられず、raw body は本文が `ResourceFunction` に残らないと判断し、`PlaceRoot::Storage(_)` と `RawBody` を含む function は store 候補から外す。nominal type も qualified definition identity が得られるまで stable type key として保存しない。
- bypass candidate connection checkpoint として、top-level `DropTraversal + ForallInitializedRange` の bypass counter を増やす前に、対応する `ResourceFunction` の body hash が作れることも確認するようにした。これにより、summary value だけは stable mirror 化できても per-summary-value key を安全に作れない関数は store 候補として観測しない。
- type boundary hash checkpoint として、`resource_summary_value_cache::type_boundary` private module を追加した。`type_parameter_boundary_hash` は `summary.type_params` の ordered boundary を使い、arity、ordinal、label、copy/clone/drop capability を含める。anonymous / bound / concrete / nominal type と、同じ stable parameter key の重複は no-store 候補にする。`generic_type_argument_hash` は順序付き argument list を stable type key で hash し、nominal identity がない argument は拒否する。
- function identity gate checkpoint として、`ResourceSummaryFunctionIdentity::from_resource_function` を追加し、canonical symbol と origin name が空でないことを bypass candidate gate でも確認するようにした。compile session 間で対応する callable 境界を特定できない function は store 候補として観測しない。
- candidate key builder checkpoint として、`resource_summary_value_cache::candidate_key` private staging module を追加した。namespace hash と source capability policy hash は型名付き wrapper で受け取り、generic type argument は `NonGeneric` / `TemplateBoundaryOnly` / `KnownInstantiation` の明示 enum にした。これにより、現行 summary が concrete call-site generic args を保持しない場合に空 slice を誤って「既知の空実引数」として扱わず、実入力が揃った場合だけ `ResourceSummaryValueCacheKey` を作る境界を固定した。
- source policy context checkpoint として、`ResourceSummaryValueCacheContext` を追加した。compiler pipeline が `ResourceSummaryCacheNamespaceKey::stable_hash` と `SourceMap::source_capability_policy_hash_for_file` から context を作り、Resource initialized check へは raw `SourceMap` ではなく `FileId -> source policy hash` の narrow context だけを渡す。context は function / block / op / terminator / nested control-flow op / match arm の source file を集約し、`Span::dummy()` や missing source policy を no-store / bypass に倒す。
- store/hit MVP checkpoint として、`ResourceSummaryValueCache` が keyable な `DropTraversal + ForallInitializedRange` stable mirror value を session map に保存し、次 compile 以降の同じ key/value を hit として観測するようにした。hit した value の Resource IR summary への逆投影はまだ行わず、統計上の再利用可能性確認に限定する。
- 同じ function/body/kind key に複数の top-level `DropTraversal + ForallInitializedRange` が存在しても順序と重複を失わないよう、complete leaf entry として保持する。hit 判定は記録開始時点で既に map にあった entry だけを対象にし、同じ summary build pass 内で store した entry を即 hit と数えない。
- reverse projection checkpoint として、store/hit 候補にする前に `ResourceSummaryStableDropTraversalForallValue` を現在 compile の `CollectionSlotLifecycleSummaryOp::DropTraversal` へ戻せることを確認するようにした。曖昧な generic boundary、範囲外 parameter index、projection layout mismatch、stride mismatch、stable type key から現在の `TypeId` へ戻せない value は no-store / bypass に倒す。これは candidate hit の安全境界であり、summary op replay と fixed-point skip は次段階に残す。
- complete leaf entry checkpoint として、cache value を個別 stable value の dedup 可能な `Vec` から、順序と重複を保持する `ResourceSummaryStableDropTraversalForallLeafEntry` に変更した。function summary 全体が top-level `DropTraversal + ForallInitializedRange` だけで、return facts が空で、summary dependency と `IndirectCall` がない場合だけ store/hit 候補にする。依存あり caller、partial summary surface、duplicate value の multiplicity について regression を追加した。fixed-point skip はまだ行わない。

## 問題

現在の API は compile call ごとに loader / source map / parse / import / typecheck / Resource IR / codegen を新規に構築する。stdlib source は bundled になっても、stdlib の parse/import/typecheck artifact と Resource IR summary template は session 間で再利用されない。

このため、同一 process 内であっても entry source の微小変更に対し、変更されていない stdlib と unchanged user functions の query result を再利用する構造が不足している。

## 影響

Web playground、Node doctest runner、selfhost compiler 開発で、実行時間ではなく compile phase が feedback loop を支配する。静的検査を強化するほど同じ stdlib graph の再検査が増え、Zenn 方針の「純粋性と静的検査を活かした performance 追求」に反する。

## 修正方針

[NEPLg2.1 compiler performance / cache design 2026-05-27](../../doc/neplg2/compiler_performance_cache_design.md) に沿って、`CompilerSession` と stdlib prechecked artifact を導入する。

MVP は次の順に進める。

1. `nepl-web` に `CompilerSession` wasm-bindgen class を公開し、Node runner が session API を優先する状態にする。
2. `nepl-core` に source text / lex / parse / import graph / type arity を query として分離する session API を追加する。現在は source arity surface cache と parsed stdlib module cache まで実装済みで、typed public surface cache は未実装。
3. Web terminal の worker を compile ごとに破棄せず、同一 WASM instance / `CompilerSession` が複数 compile にまたがって warm state を保持するようにする。これは実装済みなので、次は `CompilerSession` 側へ semantic cache を載せる。
4. `CompilerSession` に bundled stdlib の parsed module / import graph / type arity を warm state として保持する。raw parsed module、stdlib-only source import/arity surface、source-directed loader prewarm、stdlib module public surface hash、dependency aggregate public surface hash query、同一入力 compiled-output cache は実装済み。dependency aggregate query は Web prewarm hot path から外し、session-backed bundled stdlib compile path では `ResourceSummaryCacheNamespaceKey` に typed public signature hash とともに渡す。Resource summary value cache は別 field として所有し、`DropTraversal` / `ForallInitializedRange` の stable mirror value store/hit MVP まで実装済みである。現在の hit は統計観測に限定し、次は stable value を現在 compile の summary op へ逆投影する設計へ進む。
5. stdlib artifact に public signature table、trait impl index、source capability tableを持たせ、通常 compile では entry source と overlay source だけを新規処理する。
6. Resource IR summary stable mirror を namespace key + function body hash + source capability policy hash + type argument hash + summary kind/version で cache し、entry から到達する changed functions だけを再計算する。MVP は `DropTraversal` / `ForallInitializedRange` から始め、raw initialization summary 全体は store しない。
7. codegen fragment cache を function hash 単位にし、unchanged fragments を signature/index table へ再接続する。

## 完了条件

- release WASM + warm `CompilerSession` で、最小 entry source の同一 compile と 1 行変更 compile が 10ms 未満になる。
- aggregate/generic の小規模 program でも、stdlib artifact が unchanged の場合は 10ms 台を安定して維持する。
- local stdlib が release artifact より新しい場合は cache を使わず、FS stdlib override / artifact refresh に戻る。
- local stdlib content hash が release artifact の bundled stdlib hash と一致しない場合は、mtime に関係なく FS stdlib override / artifact refresh に戻る。
- raw LLVM、raw wasm direct call、indirect call、曖昧な function reference は conservative-all で検査漏れしない。
- stale diagnostic span や stale source capability が別 source へ流用されないことを regression test で固定する。

## 検証

- `trunk build --release`
- `node nodesrc/run_test.js` minimal / aggregate timing
- session API の unit test
- loader parsed stdlib cache の `FileId` 再投影 unit test
- loader parsed stdlib cache の source hash / imported type arity hint invalidation test
- source arity surface cache の source hash / root default prelude / lexer error no-preload / public re-export dependency invalidation test
- source-directed prewarm が bundled mode だけで実行され、forced / fs override では skip されることの Node runner regression test
- prewarm error が本来の compile diagnostic を置き換えないことの Node runner regression test
- user source arity surface が long-lived `LoaderSessionCache` に保存されないことの unit test
- source import surface が visibility / import clause / source order を保持し、preload path と public re-export path を同じ edge list から派生することの unit test
- root prewarm surface hash が body-only edit で変わらず、import path / import clause / relative import resolution / lexer error outcome で変わることの unit test
- module public surface hash が body-only edit で変わらず、public signature / re-export / alias target signature / private import edge / public `noshadow` で変わることの unit test
- provider prewarm 後の同一 session load で `public_surface_hash_hits` が増えることの unit test
- dependency aggregate public surface hash が re-exported stdlib dependency の body-only edit で変わらず、public signature edit で変わることの unit test
- Resource summary namespace key が public function body-only edit で変わらず、public callable type edit と dependency public surface hash input で変わることの unit test
- Resource summary value cache が entry body-only edit で compiled-output cache miss / unchanged stdlib stable summary hit を分けて観測できることの Node runner regression test
- Resource summary value cache が generic type-argument / source capability policy / target / profile / dependency public surface hash の違いで stale hit しないことの unit test
- Resource summary value cache が同じ key に複数 stable value を保存して上書きせず、2 回目の記録で各 value を hit として観測できることの unit test
- Resource summary value cache が同じ summary build pass 内で store した value を即 hit と数えないことの unit test
- forced stdlib VFS、local stdlib override、compile VFS の `/stdlib` overlay で bundled Resource summary value cache を bypass することの Node/Web regression test
- session-backed bundled stdlib compile path で loader aggregate hash が `ResourceSummaryCacheNamespaceKey` へ渡り、prewarm path と compiled-output cache hit path では dependency aggregate counter が増えないことの Node runner regression test
- non-stdlib dependency edge が bundled stdlib aggregate cache で provider read されず、bypass として観測されることの unit test
- 同一 `CompilerSession` の compiled-output cache hit が compile_ms を 10ms 未満へ下げ、source / VFS / profile / WAT comment mode の変更で stale hit しないことの Node/Web regression test
- compiled-output cache は現時点では compile VFS 全体を key に含めるため、未使用 editable `.nepl` file の変更で false miss し得る。依存 closure based key は typed public surface / import graph cache と同じ invalidation 証明を持つ段階で追加する。
- 同じ `CompilerSession` の 2 回目 prewarm reuse を `prewarm_surface_hits` で観測できることの Node runner regression test
- compile VFS に `/stdlib` overlay がある場合、bundled prewarm を skip することの Node runner regression test
- forced stdlib VFS path が session cache を使わないことの Node runner regression test
- `node nodesrc/test_run_test_compiler_session.js`
- `node nodesrc/test_playground_compiler_session_policy.js`
- stdlib artifact invalidation test
- Resource IR summary cache invalidation test
- `node nodesrc/issues.js check --dir issues`
