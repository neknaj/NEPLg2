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
| resource summary | function HIR hash + source capability hash | Resource IR summaries | function body or capability change |
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
4. Resource IR summary を function hash 単位で cache し、entry から到達する changed functions だけを再計算する。
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
