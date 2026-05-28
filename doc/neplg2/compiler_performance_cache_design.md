# NEPLg2.1 compiler performance / cache design 2026-05-27

## 位置づけ

この文書は、NEPLg2.1 移行中の compile-time performance 改善方針を固定する。対象は Rust 実装の `nepl-core`、Web/WASM wrapper の `nepl-web`、Node test runner、stdlib prelude/import graph である。

2026-05-27 に Zenn の「試作段階における開発方針」を再確認した。現段階では後方互換よりも設計の正しさを優先する。ただし静的検査の正確性、source capability、Resource IR の安全性証明を削って速度を得ることはしない。timeout 延長、検査削除、coverage 削減は performance fix として扱わない。

## 目標

- 通常の 1 program compile は 0.5 秒未満にする。
- release WASM artifact を使う warm session では、最小規模の微小再compileを 10ms 未満にする。
- stdlib は、generic の具体化以外をできるだけ事前検査済み artifact として扱い、通常 compile では link / instantiate に近い形に寄せる。
- cache は純粋な query result として扱い、入力 hash、compiler version、target、profile、stdlib artifact hash を key にする。

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

同 checkpoint では、`string/byte_index` に private `string_byte_or_invalid` と public `string_byte_is_ascii_space` を追加した。sentinel 値は byte_index module 内部に閉じ、public API は `bool` predicate のままにする。`str_trim` は loop 本体で `Option` branch を直接展開せず、この checked predicate だけを使う。これにより raw read の範囲証明は byte_index module に閉じ、Zenn 方針の Option/Result public boundary と性能上の branch 削減を両立する。

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
- `SummaryOffset::Unknown` のように、hit 後に exact place / offset を現在 compile の Resource IR context へ再投影できない projection。
- nominal type identity が unqualified name だけで、module/path/definition identity と結びついていない value。
- label 付き generic variable が function-local type parameter boundary と type-argument hash に結びついていない value。
- source capability policy hash、generic type-argument hash、target/profile、dependency public surface hash のいずれかが未確定の compile。
- `/stdlib` overlay、forced stdlib VFS、local stdlib override のように bundled stdlib proof と source が一致しない compile。

`CompilerSession.loader_cache_stats_json()` とは別に、Resource summary value cache は `resource_summary_value_hits` / `misses` / `stores` / `bypasses` と、summary kind 別の hit/store/bypass counter を持つ。compiled-output cache hit で速くなった場合と、compiled-output cache miss だが Resource summary value が hit して速くなった場合を JSON timing で分けて測定する。

2026-05-28 の implementation staging では、`ResourceSummaryValueCache` と `ResourceSummaryValueCacheStats` を `nepl-core::resource` に追加し、`CompilerSession` が `LoaderSessionCache` とは別 field として所有する形にした。`loader_cache_stats_json()` は `resource_summary_value_*` と `resource_summary_value_drop_traversal_forall_*` の counter を返す。現時点では stable mirror value の store/hit はまだ行わず、compiled-output cache と Resource summary value cache の観測境界だけを固定する。

2026-05-28 の bypass instrumentation checkpoint では、`CompilerSession` の compiled-output cache miss から実際に走る compile path だけへ `ResourceSummaryValueCache` を渡すようにした。`check_resource_initialized_moves` の既存 API は残し、session-backed compile だけが `check_resource_initialized_moves_with_summary_cache` を使う。これにより CLI や stateless Web API の挙動を変えず、session cache の観測を Web / Node の timing JSON に限定して追加できる。

この checkpoint で記録するのは、worklist 固定点が収束した後の最終 `CollectionSlotLifecycleFunctionSummary` に top-level op として残る `CollectionSlotLifecycleSummaryOp::DropTraversal` かつ `ForallInitializedRange` の候補だけである。stable mirror key/value の再投影はまだ実装していないため、候補は hit/store ではなく `resource_summary_value_drop_traversal_forall_bypasses` として数える。return path facts や `Merge` / `Loop` 内の leaf は、初期 MVP の store 対象外なのでこの counter へ含めない。これは「安全に保存できる候補が実 workload にどれだけ存在するか」を compiled-output cache とは別に測るための段階であり、`TypeId` / `Span` / `SourceMap` / typed HIR / Resource IR body を長寿命 value に保存しない方針を維持する。

2026-05-28 の stable mirror conversion checkpoint では、`DropTraversal + ForallInitializedRange` を既存 summary struct のまま cache value にせず、`ResourceSummaryStableDropTraversalForallValue` へ変換する足場を追加した。型は `TypeId` ではなく `ResourceSummaryStableTypeKey` へ変換し、無名 type variable や cycle のように arena slot へ依存する型は保存候補から外す。`SummaryPlace` / projection / symbolic offset / known i32 / `StateOnly` / `LoadedValueDrop` proof も stable mirror 型へコピーする。現 checkpoint ではまだ `BTreeMap` への store/hit は行わず、bypass counter も stable mirror へ変換できた top-level 候補だけを数える。

2026-05-28 の stable mirror split checkpoint では、stable mirror 変換を `ResourceSummaryValueCache` の sibling module ではなく、`resource_summary_value_cache::stable_mirror` private submodule へ分離した。これは cache owner の public/internal API を増やさず、store/hit 実装が入る前に「統計と map 所有」と「session-local value から stable value への変換」を別責務に分けるためである。追加 review の指摘により、`SummaryOffset::Unknown` は現時点で exact offset を再投影できないため stable mirror 変換で拒否する。nominal type key と generic variable key は、per-value key に qualified definition identity / function type parameter boundary / generic type-argument hash を含めるまで store 対象にしない。

2026-05-28 の structured key staging checkpoint では、`resource_summary_value_cache::key` private module に per-summary-value key の型だけを追加した。これは map store/hit ではなく、namespace hash、function identity、function body hash、function-local type parameter boundary hash、generic type-argument hash、source capability policy hash、summary kind/version を裸の hash blob にまとめず、field として分けたまま扱うための足場である。追加 review では、function body hash と source capability policy hash が compiler pipeline から渡るまで store/hit を入れないべきだと確認したため、この checkpoint では key 型と invalidation input regression だけを固定する。

2026-05-28 の source capability policy hash checkpoint では、`SourceCapabilities::stable_policy_hash(canonical_path, source_hash)` と `SourceMap::source_capability_policy_hash(file_id, source_hash)` を追加した。これは source capability を広く許可する query ではなく、Resource summary value key に入れるための deterministic fingerprint である。use-site proof は byte range を持つため、canonical path と source hash を必ず hash に含め、同じ proof range が別 source に流用されないようにする。現時点では function body hash と Resource summary store/hit にはまだ接続せず、path/source hash/use-site/span/order の regression だけを固定する。

2026-05-28 の function body hash staging checkpoint では、`resource_summary_value_cache::stable_type_key` を `stable_mirror` から独立させ、summary value と function body hash が同じ TypeId 安定化規則を共有できるようにした。hash writer も `stable_hash` に分け、per-summary-value key と body hash が同じ区切り付き FNV-1a 系 writer を使う。`resource_summary_value_cache::body_hash` は `ResourceFunction` の `Span` を無視し、`TypeId` を stable type key へ変換し、temporary / block id は body 内 ordinal に正規化する。`StorageId` は owner/checker state 側の割当に由来し、body だけでは安定した storage origin identity へ再投影できないため、現時点では `PlaceRoot::Storage(_)` を見たら `None` として store 候補から外す。raw body は本文が `ResourceFunction` に残らないため、raw body/source hash を key へ追加するまでは拒否する。nominal type も qualified definition identity がないため、同名別定義への stale hit を避ける目的で stable type key 変換時に拒否する。この checkpoint でも map store/hit はまだ実装しない。

同日の bypass candidate connection checkpoint では、最終 `CollectionSlotLifecycleFunctionSummary` の top-level `DropTraversal + ForallInitializedRange` を候補として数える前に、対応する `ResourceFunction` の body hash が作れることも確認するようにした。これにより、stable mirror value だけは作れても raw body / storage root / nominal type などで per-summary-value key を安全に作れない関数は、store/hit 実装前の観測 counter からも外れる。

type boundary hash checkpoint では、per-summary-value key の `type_parameter_boundary_hash` と `generic_type_argument_hash` を作る private staging module を追加した。type parameter boundary は `summary.type_params` を基準にし、unbound かつ label 付きの type variable だけを許可する。hash には arity、ordinal、label、copy/clone/drop capability を含め、同じ stable parameter key が重複した場合は再投影先が曖昧になるため拒否する。anonymous variable、bound variable、concrete type、nominal type は boundary として保存しない。generic type argument hash は順序付きの argument list として扱い、各 argument を `ResourceSummaryStableTypeKey` に変換できる場合だけ作る。nominal identity が未確定の間は store 候補が狭くなるが、同名別定義への stale hit を避けるため安全側に倒す。

必須 regression:

- 同じ entry source の 2 回目 compile は compiled-output cache hit として観測される。
- entry body-only edit で compiled-output cache は miss するが、unchanged stdlib の `DropTraversal` / `ForallInitializedRange` stable summary は hit する。
- public callable type edit、dependency public surface hash edit、generic type-argument edit、source capability policy edit、target/profile edit では stale hit しない。
- `/stdlib` overlay、forced stdlib VFS、local stdlib override では bundled stdlib Resource summary value cache を bypass する。
- cache hit value を現在 compile の Resource IR context へ再投影できない場合は miss/bypass に倒し、diagnostic span や capability proof を古い source へ流用しない。

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

MVP では次の順に実装する。

1. Web / Node に `CompilerSession` API を追加し、bundled stdlib source table を保持する。
2. Web terminal は compile ごとに worker を破棄せず、明示的な artifact refresh まで同一 worker / WASM instance / `CompilerSession` を維持する。
3. `CompilerSession` に warm parsed stdlib module cache を追加し、entry source が変わっても stdlib parse/import/type arity/typecheck artifact を再利用する。
4. Resource IR summary stable mirror を function hash 単位で cache し、entry から到達する changed functions だけを再計算する。MVP では `DropTraversal` / `ForallInitializedRange` から始め、raw initialization summary 全体は store しない。
5. codegen fragment を function hash 単位にし、unchanged functions は index と signature table だけを再接続する。
6. diagnostic rendering は最後に行い、cache には typed diagnostic enum と source span だけを保持する。

10ms 未満の対象は、CompilerSession が warm で、stdlib artifact が current で、変更が entry source の一部または小さな user module に閉じる場合である。stdlib 自体を変更した直後は artifact refresh が必要なので、この budget の対象外とする。

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
